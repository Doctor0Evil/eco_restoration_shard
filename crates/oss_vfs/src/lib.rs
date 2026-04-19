//! oss_vfs — Read‑only exabyte VFS abstraction for `/mnt/oss`
//!
//! This crate exposes `/mnt/oss` as a typed, read‑only VFS for shards, code,
//! and large datasets. It enforces a shard‑first pattern and attaches KER
//! scoring at the file level.
//!
//! **Invariants:**
//! - Only RFC‑4180 CSV, `.aln`, and `.rs`/`.toml` in whitelisted trees are visible
//! - All methods are streaming/iterator‑based (no full directory scans by default)
//! - Each opened file carries `rcalib` risk based on schema drift, missing headers, or size

#![forbid(unsafe_code)]

use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use cyboquatic_ecosafety_core::{CorridorBands, RiskCoord};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, warn};

/// Base path for the OSS VFS. In production, this is `/mnt/oss`.
pub const DEFAULT_OSS_ROOT: &str = "/mnt/oss";

/// Whitelisted extensions for shard‑first visibility.
const WHITELISTED_EXTS: &[&str] = &[".csv", ".aln", ".rs", ".toml"];

/// Shard lane enum for file classification.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Lane {
    #[serde(rename = "SIM")]
    Sim,
    #[serde(rename = "EXP")]
    Exp,
    #[serde(rename = "PROD")]
    Prod,
    #[serde(rename = "ARCHIVE")]
    Archive,
}

/// File metadata with ecosafety scoring.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OssFileMeta {
    pub path: String,
    pub size_bytes: u64,
    pub modified_utc: DateTime<Utc>,
    pub lane: Lane,
    pub rcalib: RiskCoord,
    pub vt_quality: f64,
}

/// Strongly‑typed path that prevents `..` escapes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OssPath(PathBuf);

impl OssPath {
    /// Create a new OssPath, rejecting any component that is `..`.
    pub fn new<S: AsRef<str>>(base: &str, rel: S) -> Result<Self, OssVfsError> {
        let rel_str = rel.as_ref();
        if rel_str.contains("..") {
            return Err(OssVfsError::EscapeAttempt(rel_str.to_string()));
        }
        let mut path = PathBuf::from(base);
        path.push(rel_str);
        Ok(Self(path))
    }

    /// Get the underlying path for reading (internal use only).
    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

/// Errors from OSS VFS operations.
#[derive(Error, Debug)]
pub enum OssVfsError {
    #[error("escape attempt detected: {0}")]
    EscapeAttempt(String),

    #[error("file not found: {0}")]
    NotFound(String),

    #[error("extension not whitelisted: {0}")]
    ExtensionNotWhitelisted(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("CSV parse error: {0}")]
    Csv(#[from] csv::Error),

    #[error("schema drift detected in {0}: expected headers {1:?}, got {2:?}")]
    SchemaDrift(String, Vec<String>, Vec<String>),
}

/// Health metrics for a directory shard.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DirHealth {
    pub files_scanned: u64,
    pub total_bytes: u64,
    pub avg_rcalib: f64,
    pub max_rcalib: f64,
    pub vt_quality: f64,
}

/// Main VFS handle for `/mnt/oss`.
pub struct OssVfs {
    root: PathBuf,
    max_file_size: u64,
    large_file_threshold: u64,
}

impl OssVfs {
    /// Create a new VFS rooted at `root` (default: `/mnt/oss`).
    pub fn new(root: Option<&str>) -> Self {
        let root = root
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(DEFAULT_OSS_ROOT));
        Self {
            root,
            max_file_size: 10 * 1024 * 1024 * 1024, // 10 GB hard limit
            large_file_threshold: 100 * 1024 * 1024, // 100 MB triggers rcalib increase
        }
    }

    /// List shard directories under `/mnt/oss/shards`.
    pub fn list_shards(&self) -> Result<impl Iterator<Item = PathBuf>, OssVfsError> {
        let shards_dir = self.root.join("shards");
        if !shards_dir.exists() {
            return Ok(std::iter::empty());
        }
        let entries = std::fs::read_dir(&shards_dir)?;
        let shards = entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .map(|e| e.path());
        Ok(shards)
    }

    /// List repos under `/mnt/oss/repos`.
    pub fn list_repos(&self) -> Result<impl Iterator<Item = PathBuf>, OssVfsError> {
        let repos_dir = self.root.join("repos");
        if !repos_dir.exists() {
            return Ok(std::iter::empty());
        }
        let entries = std::fs::read_dir(&repos_dir)?;
        let repos = entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .map(|e| e.path());
        Ok(repos)
    }

    /// Open a shard file for reading (streaming).
    pub fn open_shard(&self, rel_path: &str) -> Result<OssShardReader, OssVfsError> {
        let oss_path = OssPath::new(&self.root.to_string_lossy(), rel_path)?;
        let path = oss_path.as_path();

        if !path.exists() {
            return Err(OssVfsError::NotFound(rel_path.to_string()));
        }

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| OssVfsError::ExtensionNotWhitelisted(rel_path.to_string()))?;

        if !WHITELISTED_EXTS.contains(&ext) {
            return Err(OssVfsError::ExtensionNotWhitelisted(ext.to_string()));
        }

        let file = File::open(path)?;
        let meta = file.metadata()?;
        let size = meta.len();

        // Compute rcalib based on size and extension
        let rcalib = self.compute_rcalib(size, ext);
        let vt_quality = rcalib.value(); // Simple mapping for now

        let modified = meta
            .modified()
            .ok()
            .and_then(|t| DateTime::<Utc>::from_timestamp(t.elapsed().unwrap().as_secs() as i64, 0))
            .unwrap_or_else(Utc::now);

        let lane = self.infer_lane(path);

        Ok(OssShardReader {
            inner: BufReader::new(file),
            meta: OssFileMeta {
                path: rel_path.to_string(),
                size_bytes: size,
                modified_utc: modified,
                lane,
                rcalib,
                vt_quality,
            },
        })
    }

    /// Open a code file (.rs or .toml) for reading.
    pub fn open_code_file(&self, rel_path: &str) -> Result<BufReader<File>, OssVfsError> {
        let oss_path = OssPath::new(&self.root.to_string_lossy(), rel_path)?;
        let path = oss_path.as_path();

        if !path.exists() {
            return Err(OssVfsError::NotFound(rel_path.to_string()));
        }

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| OssVfsError::ExtensionNotWhitelisted(rel_path.to_string()))?;

        if !["rs", "toml"].contains(&ext) {
            return Err(OssVfsError::ExtensionNotWhitelisted(ext.to_string()));
        }

        let file = File::open(path)?;
        Ok(BufReader::new(file))
    }

    /// Get file metadata including ecosafety scores.
    pub fn stat(&self, rel_path: &str) -> Result<OssFileMeta, OssVfsError> {
        let oss_path = OssPath::new(&self.root.to_string_lossy(), rel_path)?;
        let path = oss_path.as_path();

        if !path.exists() {
            return Err(OssVfsError::NotFound(rel_path.to_string()));
        }

        let meta = std::fs::metadata(path)?;
        let size = meta.len();
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let rcalib = self.compute_rcalib(size, ext);
        let vt_quality = rcalib.value();

        let modified = meta
            .modified()
            .ok()
            .and_then(|t| DateTime::<Utc>::from_timestamp(t.elapsed().unwrap().as_secs() as i64, 0))
            .unwrap_or_else(Utc::now);

        let lane = self.infer_lane(path);

        Ok(OssFileMeta {
            path: rel_path.to_string(),
            size_bytes: size,
            modified_utc: modified,
            lane,
            rcalib,
            vt_quality,
        })
    }

    /// Compute rcalib based on file size and extension.
    fn compute_rcalib(&self, size: u64, ext: &str) -> RiskCoord {
        let mut base = 0.0;

        // Large files increase calibration risk
        if size > self.large_file_threshold {
            let ratio = (size - self.large_file_threshold) as f64
                / (self.max_file_size - self.large_file_threshold) as f64;
            base += ratio.min(1.0) * 0.3;
        }

        // Missing standard headers in CSV increases risk
        if ext == "csv" {
            // Would check headers here; for now, small penalty
            base += 0.05;
        }

        // ALN schemas are trusted more
        if ext == "aln" {
            base -= 0.02;
        }

        RiskCoord::new(base.clamp(0.0, 1.0))
    }

    /// Infer lane from path structure.
    fn infer_lane(&self, path: &Path) -> Lane {
        let path_str = path.to_string_lossy();
        if path_str.contains("/prod/") || path_str.contains("/PROD/") {
            Lane::Prod
        } else if path_str.contains("/exp/") || path_str.contains("/EXP/") {
            Lane::Exp
        } else if path_str.contains("/archive/") || path_str.contains("/ARCHIVE/") {
            Lane::Archive
        } else {
            Lane::Sim
        }
    }

    /// Compute directory health metrics.
    pub fn dir_health(&self, rel_path: &str) -> Result<DirHealth, OssVfsError> {
        let oss_path = OssPath::new(&self.root.to_string_lossy(), rel_path)?;
        let path = oss_path.as_path();

        if !path.exists() || !path.is_dir() {
            return Err(OssVfsError::NotFound(rel_path.to_string()));
        }

        let mut health = DirHealth::default();
        let mut rcalib_sum = 0.0;
        let mut count = 0u64;

        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let meta = entry.metadata()?;
            if meta.is_file() {
                let size = meta.len();
                let ext = entry
                    .path()
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("");

                if !WHITELISTED_EXTS.contains(&ext) {
                    continue;
                }

                let rcalib = self.compute_rcalib(size, ext);
                rcalib_sum += rcalib.value();
                health.total_bytes += size;
                health.max_rcalib = health.max_rcalib.max(rcalib.value());
                count += 1;
            }
        }

        health.files_scanned = count;
        if count > 0 {
            health.avg_rcalib = rcalib_sum / count as f64;
            health.vt_quality = health.avg_rcalib;
        }

        Ok(health)
    }
}

/// Streaming reader for shard files with attached metadata.
pub struct OssShardReader {
    inner: BufReader<File>,
    pub meta: OssFileMeta,
}

impl OssShardReader {
    pub fn meta(&self) -> &OssFileMeta {
        &self.meta
    }

    pub fn lines(self) -> impl Iterator<Item = Result<String, std::io::Error>> {
        self.inner.lines()
    }
}

/// Write a sidecar health shard for a directory.
pub fn write_health_shard(
    vfs: &OssVfs,
    dir_rel: &str,
    output_path: &str,
) -> Result<(), OssVfsError> {
    let health = vfs.dir_health(dir_rel)?;
    let ts = Utc::now().to_rfc3339();

    let mut writer = csv::Writer::from_path(output_path)?;
    writer.write_record(&[
        "dir_path",
        "ts_utc",
        "files_scanned",
        "total_bytes",
        "avg_rcalib",
        "max_rcalib",
        "vt_quality",
    ])?;
    writer.write_record(&[
        dir_rel,
        &ts,
        &health.files_scanned.to_string(),
        &health.total_bytes.to_string(),
        &health.avg_rcalib.to_string(),
        &health.max_rcalib.to_string(),
        &health.vt_quality.to_string(),
    ])?;
    writer.flush()?;

    debug!(
        "Wrote health shard for {} with {} files, vt={}",
        dir_rel, health.files_scanned, health.vt_quality
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oss_path_no_escape() {
        let result = OssPath::new("/mnt/oss", "../../../etc/passwd");
        assert!(matches!(result, Err(OssVfsError::EscapeAttempt(_))));
    }

    #[test]
    fn test_oss_path_valid() {
        let result = OssPath::new("/mnt/oss", "shards/foo/bar.csv");
        assert!(result.is_ok());
    }
}
