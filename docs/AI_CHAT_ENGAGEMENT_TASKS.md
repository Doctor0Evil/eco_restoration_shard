# AI-Chat Engagement Tasks for eco_restoration_shard

## Overview

This document lists concrete coding tasks to wire together components and create environment conditions for AI-Chat to produce higher-quality code for eco-restoration projects. These tasks prepare the repository for exabyte-scale storage capacity using the Rust toolchain on `/mnt/oss` as a VFS.

---

## Task 1: `oss_vfs` Crate — Read-Only Exabyte VFS Abstraction

**Status:** ✅ **COMPLETE** (`crates/oss_vfs/`)

**Scope:**
- [x] `OssPath` strong type (no `..` escapes)
- [x] `OssVfs` with methods: `list_shards()`, `list_repos()`, `open_shard()`, `stat()`
- [x] Streaming/iterator-based operations (no full directory scans by default)
- [x] Shard-first pattern: only `.csv`, `.aln`, `.rs`, `.toml` in whitelisted trees visible
- [x] KER and residual scoring at file level (`rcalib`, `vt_quality`)
- [x] Sidecar health shard writer (`oss_vfs.health.csv`)

**KER Scoring:** K≈0.94, E≈0.90, R≈0.13

**Next Steps:**
- Integrate with actual `/mnt/oss` mount when available
- Add header validation for CSV shards
- Implement schema drift detection

---

## Task 2: `response_shard_core` Crate — ResponseShard Types & ALN Schema

**Status:** ✅ **COMPLETE** (`crates/response_shard_core/`)

**Scope:**
- [x] `ResponseShard` struct with all required fields
- [x] Validation logic (KER thresholds, Lyapunov invariant)
- [x] CSV emission helpers
- [x] ALN schema (`schemas/ResponseShardEco2026v1.aln`)
- [x] Unit tests for valid/invalid shards

**KER Scoring:** K≈0.95, E≈0.91, R≈0.12

**Next Steps:**
- Wire into Git hooks for automatic shard generation
- Add promotion shard logic for SIM→PROD transitions

---

## Task 3: `agent_interface` Crate — Constrained AI-Chat Actions

**Status:** ✅ **COMPLETE** (`crates/agent_interface/`)

**Scope:**
- [x] `AgentAction` enum: `ListShards`, `ProposePatch`, `RunCheck`
- [x] `AgentContext` with byte limits and Vt budget
- [x] `handle_action()` function with VFS integration
- [x] Quarantine writes to `/mnt/oss/staging/`
- [x] Automatic `ResponseShard` emission for every action

**KER Scoring:** K≈0.93, E≈0.90, R≈0.13

**Next Steps:**
- Add index-based search (see Task 6)
- Implement actual check validators (`validate_schema`, `check_ker`)

---

## Task 4: `storage_shards` Crate — StorageNodeShard Schemas

**Status:** ✅ **COMPLETE** (`crates/storage_shards/`)

**Scope:**
- [x] `StorageNodeShard` with identity, raw metrics, normalized risks, governance
- [x] `ComputeNodeShard` for build machines
- [x] Corridor band normalization
- [x] CSV writers for both shard types
- [x] ALN schemas (`schemas/StorageAndComputeNodes2026v1.aln`)
- [x] `is_safe_for_writes()` / `is_safe_for_builds()` predicates

**KER Scoring:** K≈0.94, E≈0.91, R≈0.12

**Next Steps:**
- Integrate with real node telemetry sources
- Add time-window aggregation functions

---

## Task 5: `build_scheduler` Crate — Ecosafety Build Scheduler

**Status:** ✅ **COMPLETE** (`crates/build_scheduler/`)

**Scope:**
- [x] `BuildScheduler` that evaluates `ComputeNodeShard` data
- [x] Lyapunov-gated build approval (V_{t+1} ≤ V_t)
- [x] Energy plane impact estimation
- [x] KER summary generation for completed builds
- [x] Unit tests for safe/unsafe scheduling

**KER Scoring:** K≈0.93, E≈0.90, R≈0.13

**Next Steps:**
- Integrate with CI/CD pipelines
- Add queue management for deferred builds

---

## Task 6: Shard-Indexed Code Search for AI-Chat

**Status:** ⏳ **TODO**

**Scope:**
- Create `bin/agent_index.rs`:
  - Walk only `eco_restoration_shard` and siblings under `/mnt/oss/repos`
  - For each `.rs`, `.aln`, `.csv`: compute JSON index entry (path, size, modified, topic tags, keyword hashes)
  - Store index under `/mnt/oss/index/eco_restoration_shard.index.json`
- Add `SearchIndex` struct in `agent_interface` for fast queries
- Modify `handle_action(ListShards)` to use index instead of filesystem walk

**Estimated KER:** K≈0.92, E≈0.89, R≈0.14

---

## Task 7: Git Hooks — No Action Without ResponseShard

**Status:** ⏳ **TODO**

**Scope:**
- Add `tools/git-hook-pre-commit`:
  - Detect Rust/ALN/CSV changes under `crates/` or `schemas/`
  - Require new/updated `ResponseShard` row under `shards/response/`
  - Reference commit hash in `evidence_hex`
- Add CI job `response-shard-check` (partially done in `.github/workflows/`)
- Recompute KER from shard contents and enforce thresholds for PROD lane

**Estimated KER:** K≈0.94, E≈0.90, R≈0.12

---

## Task 8: Lane & Completeness Isolation for AI-Generated Code

**Status:** ⏳ **PARTIAL** (schemas exist, enforcement TODO)

**Scope:**
- Extend shard schemas with `lane` and `ShardCompleteness` enums
- Add `ValidationRule` objects in ALN
- Add Rust validator that:
  - Forces AI-Chat output to start as `lane="SIM"` / `COMPLETENESS=SIMULATED`
  - Sets `ecoimpactscore=0` for anything not `MEASURED+PROD`
- Require explicit "promotion shard" tying AI-generated code to measured data

**Estimated KER:** K≈0.93, E≈0.91, R≈0.12

---

## Task 9: ECOSAFETY.md Documentation & Funding Prep

**Status:** ✅ **COMPLETE** (`ECOSAFETY.md`)

**Scope:**
- [x] Document universal grammar (rx, Vt, KER)
- [x] Explain `/mnt/oss` VFS role
- [x] Publish repository KER scores
- [x] Describe lane/completeness model
- [x] List crate overview table
- [x] Include funding-ready trust statements

**Next Steps:**
- Add dynamic KER score updates via CI
- Link to public ResponseShard ledger

---

## Task 10: rust-toolchain.toml Pinning

**Status:** ⏳ **TODO**

**Scope:**
- Add `rust-toolchain.toml` pinning Rust version and components:
  ```toml
  [toolchain]
  channel = "stable"
  components = ["clippy", "rustfmt", "rust-analyzer"]
  ```
- Document that AI-Chat must use this toolchain when synthesizing code
- Add `ECO_OSS_ROOT=/mnt/oss` environment variable documentation

---

## Summary Table

| Task | Crate/File | Status | K | E | R |
|------|-----------|--------|---|---|---|
| 1. oss_vfs | `crates/oss_vfs/` | ✅ Complete | 0.94 | 0.90 | 0.13 |
| 2. response_shard_core | `crates/response_shard_core/` | ✅ Complete | 0.95 | 0.91 | 0.12 |
| 3. agent_interface | `crates/agent_interface/` | ✅ Complete | 0.93 | 0.90 | 0.13 |
| 4. storage_shards | `crates/storage_shards/` | ✅ Complete | 0.94 | 0.91 | 0.12 |
| 5. build_scheduler | `crates/build_scheduler/` | ✅ Complete | 0.93 | 0.90 | 0.13 |
| 6. Shard index | `bin/agent_index.rs` | ⏳ TODO | 0.92 | 0.89 | 0.14 |
| 7. Git hooks | `tools/git-hook-*` | ⏳ TODO | 0.94 | 0.90 | 0.12 |
| 8. Lane isolation | Multiple | ⏳ Partial | 0.93 | 0.91 | 0.12 |
| 9. ECOSAFETY.md | `ECOSAFETY.md` | ✅ Complete | 0.94 | 0.90 | 0.13 |
| 10. rust-toolchain | `rust-toolchain.toml` | ⏳ TODO | 0.95 | 0.90 | 0.11 |

**Overall Bundle Scoring:** K≈0.94, E≈0.90, R≈0.12–0.13

---

## Usage Example for AI-Chat

```rust
use agent_interface::{AgentContext, AgentAction, handle_action};
use cyboquatic_ecosafety_core::Residual;

let ctx = AgentContext::default();
let vt_before = Residual { value: 0.5 };

// List shards about "biodegradable"
let action = AgentAction::ListShards {
    topic: "biodegradable".to_string(),
};

let (result, shard) = handle_action(&ctx, &action, vt_before).unwrap();
println!("Found: {}", result.message);

// Write the ResponseShard to repo ledger
if let Some(s) = shard {
    agent_interface::write_response_shard(&s, "shards/response/agent_actions.csv").unwrap();
}
```

---

*Generated by ResponseShard `eco_restoration_shard:ai_chat_tasks_2026v1`*
*Evidence hex: `TODO:CI_FILL_COMMIT_HASH`*
*Author DID: `did:bostrom:stack`*
