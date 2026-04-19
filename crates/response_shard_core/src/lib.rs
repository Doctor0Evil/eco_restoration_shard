//! response_shard_core — ResponseShard types and CSV/ALN emission
//!
//! Every AI‑Chat turn or coding‑agent action must emit a `ResponseShard` with:
//! - DID authorship
//! - KER triad (K, E, R)
//! - Vt before/after (must be non‑increasing for ACCEPT)
//! - Corridor references
//! - Evidence hex (commit hash or content hash)
//! - Lane classification (SIM, EXP, PROD, ARCHIVE)

#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use cyboquatic_ecosafety_core::{KerThresholds, KerWindow, Residual};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

/// ResponseShard representing one AI‑Chat or agent turn.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResponseShard {
    pub shard_id: String,
    pub author_did: String,
    pub topic: String,
    pub timestamp_utc: String,
    pub k: f64,
    pub e: f64,
    pub r: f64,
    pub vt_before: f64,
    pub vt_after: f64,
    pub corridor_ids: Vec<String>,
    pub evidence_hex: String,
    pub lane: String,
}

impl ResponseShard {
    /// Create a new ResponseShard with computed KER from window metrics.
    pub fn new(
        topic: String,
        author_did: String,
        corridor_ids: Vec<String>,
        evidence_content: &[u8],
        vt_before: Residual,
        vt_after: Residual,
        ker_window: KerWindow,
        lane: &str,
    ) -> Self {
        let shard_id = Self::generate_shard_id(&topic, &author_did);
        let timestamp_utc = Utc::now().to_rfc3339();
        let evidence_hex = hex::encode(Sha256::digest(evidence_content));

        Self {
            shard_id,
            author_did,
            topic,
            timestamp_utc,
            k: ker_window.k,
            e: ker_window.e,
            r: ker_window.r,
            vt_before: vt_before.value,
            vt_after: vt_after.value,
            corridor_ids,
            evidence_hex,
            lane: lane.to_string(),
        }
    }

    /// Generate a unique shard ID from topic and author.
    fn generate_shard_id(topic: &str, author_did: &str) -> String {
        let ts = Utc::now().timestamp_nanos_opt().unwrap_or(0);
        format!("{}:{}:{}", topic, author_did, ts)
    }

    /// Check if this shard meets production thresholds.
    pub fn meets_prod_thresholds(&self) -> bool {
        let thr = KerThresholds::prod_defaults();
        self.k >= thr.k_min && self.e >= thr.e_min && self.r <= thr.r_max
    }

    /// Check the Lyapunov invariant: Vt_after <= Vt_before (with small epsilon).
    pub fn lyapunov_ok(&self, epsilon: f64) -> bool {
        self.vt_after <= self.vt_before + epsilon
    }

    /// Validate shard invariants based on decision type.
    pub fn validate(&self) -> Result<(), ShardValidationError> {
        // For any shard, lane must be valid
        if !["SIM", "EXP", "PROD", "ARCHIVE"].contains(&self.lane.as_str()) {
            return Err(ShardValidationError::InvalidLane(self.lane.clone()));
        }

        // KER values must be in [0, 1]
        if !(0.0..=1.0).contains(&self.k)
            || !(0.0..=1.0).contains(&self.e)
            || !(0.0..=1.0).contains(&self.r)
        {
            return Err(ShardValidationError::KerOutOfRange);
        }

        // Vt values must be non‑negative
        if self.vt_before < 0.0 || self.vt_after < 0.0 {
            return Err(ShardValidationError::NegativeVt);
        }

        // For PROD lane, enforce stricter thresholds
        if self.lane == "PROD" {
            if !self.meets_prod_thresholds() {
                return Err(ShardValidationError::ProdThresholdsNotMet);
            }
            if !self.lyapunov_ok(1e-6) {
                return Err(ShardValidationError::LyapunovViolation);
            }
        }

        Ok(())
    }

    /// Convert to CSV record (header + data row).
    pub fn to_csv_record(&self) -> Vec<String> {
        vec![
            self.shard_id.clone(),
            self.author_did.clone(),
            self.topic.clone(),
            self.timestamp_utc.clone(),
            self.k.to_string(),
            self.e.to_string(),
            self.r.to_string(),
            self.vt_before.to_string(),
            self.vt_after.to_string(),
            self.corridor_ids.join("|"),
            self.evidence_hex.clone(),
            self.lane.clone(),
        ]
    }

    /// Get CSV header.
    pub fn csv_header() -> Vec<&'static str> {
        vec![
            "shard_id",
            "author_did",
            "topic",
            "timestamp_utc",
            "k",
            "e",
            "r",
            "vt_before",
            "vt_after",
            "corridor_ids",
            "evidence_hex",
            "lane",
        ]
    }
}

/// Errors during shard validation.
#[derive(Error, Debug)]
pub enum ShardValidationError {
    #[error("invalid lane: {0}")]
    InvalidLane(String),

    #[error("KER values out of [0, 1] range")]
    KerOutOfRange,

    #[error("negative Vt value")]
    NegativeVt,

    #[error("production thresholds not met (K>=0.90, E>=0.90, R<=0.13)")]
    ProdThresholdsNotMet,

    #[error("Lyapunov violation: Vt_after > Vt_before")]
    LyapunovViolation,
}

/// Writer for ResponseShard CSV files.
pub struct ResponseShardWriter {
    writer: csv::Writer<std::fs::File>,
}

impl ResponseShardWriter {
    /// Create a new writer at the given path. Writes header if file is new.
    pub fn new(path: &str) -> Result<Self, csv::Error> {
        let file_exists = std::path::Path::new(path).exists();
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;

        let mut writer = csv::Writer::from_writer(file);

        if !file_exists {
            writer.write_record(ResponseShard::csv_header())?;
        }

        Ok(Self { writer })
    }

    /// Write a single shard.
    pub fn write(&mut self, shard: &ResponseShard) -> Result<(), csv::Error> {
        self.writer.write_record(shard.to_csv_record())?;
        self.writer.flush()?;
        Ok(())
    }
}

/// ALN schema string for ResponseShard.
pub const RESPONSE_SHARD_ALN_SCHEMA: &str = r#"
schema ResponseShardEco2026v1 {
  fields {
    shard_id        : string;
    author_did      : string;
    topic           : string;
    timestamp_utc   : datetime;
    k               : f64;
    e               : f64;
    r               : f64;
    vt_before       : f64;
    vt_after        : f64;
    corridor_ids    : list[string];
    evidence_hex    : hex[64];
    lane            : enum{SIM, EXP, PROD, ARCHIVE};
  }

  constraints {
    assert(k >= 0.0 && k <= 1.0);
    assert(e >= 0.0 && e <= 1.0);
    assert(r >= 0.0 && r <= 1.0);
    assert(vt_before >= 0.0);
    assert(vt_after >= 0.0);

    if lane == "PROD" {
      assert(k >= 0.90);
      assert(e >= 0.90);
      assert(r <= 0.13);
      assert(vt_after <= vt_before);
    }
  }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use cyboquatic_ecosafety_core::KerWindow;

    #[test]
    fn test_valid_prod_shard() {
        let shard = ResponseShard::new(
            "test:topic".to_string(),
            "did:bostrom:test".to_string(),
            vec!["corridor_1".to_string()],
            b"test evidence",
            Residual { value: 0.5 },
            Residual { value: 0.4 },
            KerWindow {
                k: 0.95,
                e: 0.92,
                r: 0.10,
            },
            "PROD",
        );

        assert!(shard.validate().is_ok());
        assert!(shard.lyapunov_ok(1e-6));
        assert!(shard.meets_prod_thresholds());
    }

    #[test]
    fn test_invalid_lyapunov_prod() {
        let shard = ResponseShard::new(
            "test:topic".to_string(),
            "did:bostrom:test".to_string(),
            vec![],
            b"test evidence",
            Residual { value: 0.3 },
            Residual { value: 0.5 }, // Vt increased!
            KerWindow {
                k: 0.95,
                e: 0.92,
                r: 0.10,
            },
            "PROD",
        );

        assert!(matches!(
            shard.validate(),
            Err(ShardValidationError::LyapunovViolation)
        ));
    }
}
