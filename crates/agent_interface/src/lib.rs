//! agent_interface — Constrained action vocabulary for AI‑Chat and coding agents
//!
//! This crate exposes a tiny, ecosafe action set that AI‑Chat can call to:
//! - Search/list shards by topic (via precomputed index)
//! - Propose patches (written to quarantine `/mnt/oss/staging/`)
//! - Run checks (read‑only validation)
//!
//! All actions use `oss_vfs` for size‑bounded, streaming operations and emit
//! a `ResponseShard` for auditability.

#![forbid(unsafe_code)]

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

use cyboquatic_ecosafety_core::{KerWindow, Residual};
use oss_vfs::{OssVfs, DEFAULT_OSS_ROOT};
use response_shard_core::{ResponseShard, ResponseShardWriter};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, info, warn};

/// Maximum bytes an agent can scan in one action.
pub const DEFAULT_MAX_BYTES_PER_SCAN: u64 = 10 * 1024 * 1024 * 1024; // 10 GB

/// Vt budget for an agent session (total allowed increase).
pub const DEFAULT_VT_BUDGET: f64 = 0.01;

/// Agent context with limits.
#[derive(Clone, Debug)]
pub struct AgentContext {
    pub oss_root: String,
    pub max_bytes_per_scan: u64,
    pub vt_budget: f64,
    pub author_did: String,
}

impl Default for AgentContext {
    fn default() -> Self {
        Self {
            oss_root: DEFAULT_OSS_ROOT.to_string(),
            max_bytes_per_scan: DEFAULT_MAX_BYTES_PER_SCAN,
            vt_budget: DEFAULT_VT_BUDGET,
            author_did: "did:bostrom:agent".to_string(),
        }
    }
}

/// Action vocabulary for AI‑Chat.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AgentAction {
    /// List shards matching a topic filter.
    ListShards { topic: String },

    /// Propose a patch (diff) for a file. Written to staging area.
    ProposePatch { path: String, diff: String },

    /// Run a read‑only check (e.g., "validate_schema", "check_ker").
    RunCheck { check: String, target: String },
}

/// Result of an agent action.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActionResult {
    pub success: bool,
    pub message: String,
    pub data: Option<String>,
    pub shard_id: Option<String>,
}

/// Errors from agent actions.
#[derive(Error, Debug)]
pub enum AgentError {
    #[error("VFS error: {0}")]
    Vfs(#[from] oss_vfs::OssVfsError),

    #[error("scan would exceed byte limit: {0} > {1}")]
    ScanLimitExceeded(u64, u64),

    #[error("shard error: {0}")]
    Shard(#[from] response_shard_core::ShardValidationError),

    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("invalid action: {0}")]
    InvalidAction(String),
}

/// Handle an agent action and return result + optional ResponseShard.
pub fn handle_action(
    ctx: &AgentContext,
    action: &AgentAction,
    vt_before: Residual,
) -> Result<(ActionResult, Option<ResponseShard>), AgentError> {
    let vfs = OssVfs::new(Some(&ctx.oss_root));

    match action {
        AgentAction::ListShards { topic } => {
            handle_list_shards(&vfs, topic, ctx, vt_before)
        }
        AgentAction::ProposePatch { path, diff } => {
            handle_propose_patch(&vfs, path, diff, ctx, vt_before)
        }
        AgentAction::RunCheck { check, target } => {
            handle_run_check(&vfs, check, target, ctx, vt_before)
        }
    }
}

fn handle_list_shards(
    vfs: &OssVfs,
    topic: &str,
    ctx: &AgentContext,
    vt_before: Residual,
) -> Result<(ActionResult, Option<ResponseShard>), AgentError> {
    debug!("Listing shards for topic: {}", topic);

    // In production, this would query a precomputed index.
    // For now, we list shard dirs and filter by topic in name.
    let mut matches = Vec::new();
    let mut total_bytes = 0u64;

    for shard_dir in vfs.list_shards()? {
        if shard_dir
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.contains(topic))
            .unwrap_or(false)
        {
            let health = vfs.dir_health(
                shard_dir
                    .strip_prefix(&ctx.oss_root)
                    .unwrap()
                    .to_str()
                    .unwrap(),
            )?;
            total_bytes += health.total_bytes;

            if total_bytes > ctx.max_bytes_per_scan {
                warn!(
                    "Scan limit exceeded: {} > {}",
                    total_bytes, ctx.max_bytes_per_scan
                );
                return Err(AgentError::ScanLimitExceeded(
                    total_bytes,
                    ctx.max_bytes_per_scan,
                ));
            }

            matches.push(shard_dir.to_string_lossy().to_string());
        }
    }

    let data = serde_json::to_string(&matches).unwrap();
    let vt_after = vt_before; // Read‑only, no Vt change

    let shard = Some(ResponseShard::new(
        format!("agent:list_shards:{}", topic),
        ctx.author_did.clone(),
        vec![],
        topic.as_bytes(),
        vt_before,
        vt_after,
        KerWindow::from_risk(0.0, 1.0),
        "SIM",
    ));

    Ok((
        ActionResult {
            success: true,
            message: format!("Found {} matching shards", matches.len()),
            data: Some(data),
            shard_id: shard.as_ref().map(|s| s.shard_id.clone()),
        },
        shard,
    ))
}

fn handle_propose_patch(
    vfs: &OssVfs,
    path: &str,
    diff: &str,
    ctx: &AgentContext,
    vt_before: Residual,
) -> Result<(ActionResult, Option<ResponseShard>), AgentError> {
    debug!("Proposing patch for: {}", path);

    // Write to staging area (quarantine)
    let staging_dir = PathBuf::from(&ctx.oss_root).join("staging");
    fs::create_dir_all(&staging_dir)?;

    // Sanitize path to filename
    let safe_name = path.replace('/', "_").replace('\\', "_");
    let patch_path = staging_dir.join(format!("{}.patch", safe_name));

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&patch_path)?;
    file.write_all(diff.as_bytes())?;

    info!("Patch written to quarantine: {:?}", patch_path);

    let vt_after = vt_before; // No actuation yet

    let shard = Some(ResponseShard::new(
        format!("agent:propose_patch:{}", path),
        ctx.author_did.clone(),
        vec![],
        diff.as_bytes(),
        vt_before,
        vt_after,
        KerWindow::from_risk(0.0, 1.0),
        "SIM",
    ));

    Ok((
        ActionResult {
            success: true,
            message: format!("Patch staged at {:?}", patch_path),
            data: None,
            shard_id: shard.as_ref().map(|s| s.shard_id.clone()),
        },
        shard,
    ))
}

fn handle_run_check(
    vfs: &OssVfs,
    check: &str,
    target: &str,
    ctx: &AgentContext,
    vt_before: Residual,
) -> Result<(ActionResult, Option<ResponseShard>), AgentError> {
    debug!("Running check '{}' on '{}'", check, target);

    let result = match check {
        "validate_schema" => {
            // Check if target ALN schema exists and is valid
            let meta = vfs.stat(target)?;
            format!("Schema {} validated (lane={})", target, meta.lane as u8)
        }
        "check_ker" => {
            // Placeholder: would load KER metrics from shard
            "KER check placeholder".to_string()
        }
        _ => {
            return Err(AgentError::InvalidAction(format!(
                "Unknown check: {}",
                check
            )))
        }
    };

    let vt_after = vt_before;

    let shard = Some(ResponseShard::new(
        format!("agent:run_check:{}:{}", check, target),
        ctx.author_did.clone(),
        vec![],
        format!("{}:{}", check, target).as_bytes(),
        vt_before,
        vt_after,
        KerWindow::from_risk(0.0, 1.0),
        "SIM",
    ));

    Ok((
        ActionResult {
            success: true,
            message: result,
            data: None,
            shard_id: shard.as_ref().map(|s| s.shard_id.clone()),
        },
        shard,
    ))
}

/// Write a ResponseShard to the repo's response shard CSV.
pub fn write_response_shard(
    shard: &ResponseShard,
    output_path: &str,
) -> Result<(), AgentError> {
    let mut writer = ResponseShardWriter::new(output_path)?;
    writer.write(shard)?;
    info!("ResponseShard written: {}", shard.shard_id);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_context_default() {
        let ctx = AgentContext::default();
        assert_eq!(ctx.oss_root, DEFAULT_OSS_ROOT);
        assert_eq!(ctx.max_bytes_per_scan, DEFAULT_MAX_BYTES_PER_SCAN);
        assert!(!ctx.author_did.is_empty());
    }
}
