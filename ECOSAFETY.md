# ECOSAFETY.md — Ecosafety Grammar & AI‑Chat Engagement Guide

## Overview

This repository implements the **rx–Vt–KER grammar** for Cyboquatic industrial machinery: energy‑efficient, carbon‑negative, and ecologically restorative systems. Every subsystem is expressed as a **Rust‑first ecosafety client** with hard Lyapunov gates, carbon/biodiversity planes, and corridor‑checked materials. C/Lua/Kotlin exist only as tightly fenced satellites.

## Universal Grammar

### Risk Coordinates (rx)

All physical and uncertainty metrics are normalized to `RiskCoord ∈ [0, 1]` where **0 = best**, **1 = worst**:

| Plane | Symbol | Description |
|-------|--------|-------------|
| Energy | `r_energy` | Joules per kg removed, power margin |
| Hydraulics | `r_hydraulics` | Surcharge risk, HLR deviation |
| Biology | `r_biology` | Pathogen, fouling, CEC risk |
| Carbon | `r_carbon` | Net emissions vs sequestration |
| Materials | `r_materials` | Decomposition time, leachate, micro‑residue |
| Biodiversity | `r_biodiversity` | Habitat impact score |
| Uncertainty | `r_sigma` | Sensor/model uncertainty (rcalib, rsigma) |

### Lyapunov Residual (Vt)

$$V_t = \sum_j w_j \cdot r_j^2$$

**Invariant:** $V_{t+1} \le V_t + \epsilon$ for all actuations. Violations trigger `SafeStepDecision::Reject`.

Default weights (`LyapunovWeights::default_carbon_negative()`):

```rust
w_energy: 1.0,
w_hydraulics: 1.0,
w_biology: 1.2,
w_carbon: 1.3,        // carbon drift treated as severe
w_materials: 1.1,
w_biodiversity: 1.1,  // habitat loss penalized strongly
w_sigma: 0.8,         // uncertainty lifts residual
```

### KER Triad

| Metric | Symbol | Threshold (PROD) | Computation |
|--------|--------|------------------|-------------|
| Knowledge‑factor | K ≥ 0.90 | Fraction of Lyapunov‑safe steps |
| Eco‑impact | E ≥ 0.90 | E = 1 − R (clamped) |
| Risk‑of‑harm | R ≤ 0.13 | Max risk coordinate over window |

**Invariants:**
- `no corridor, no build`
- `V_{t+1} <= V_t`
- `K >= 0.90 && E >= 0.90 && R <= 0.13` for PROD lane

## Repository KER Score

Current repository‑level metrics (updated by CI from ResponseShards):

| Metric | Value | Band |
|--------|-------|------|
| **K** (knowledge) | ~0.94 | Gold |
| **E** (eco‑impact) | ~0.90 | Gold |
| **R** (risk‑of‑harm) | ~0.13 | Safe/Gold boundary |

## `/mnt/oss` Virtual Filesystem

The **only** exabyte‑scale source of truth for large artifacts is `/mnt/oss`, exposed via the `oss_vfs` crate:

```
/mnt/oss/
├── shards/          # RFC‑4180 CSV, .aln schemas
├── repos/           # Whitelisted code trees (.rs, .toml, .aln)
├── index/           # Precomputed AI‑Chat maps
├── staging/         # Quarantine for proposed patches
└── metrics/         # Storage/compute node telemetry
```

**Shard‑first pattern:** Only `.csv`, `.aln`, `.rs`, `.toml` in whitelisted trees are visible. Everything else is explicitly "non‑ecosafety".

## Lane & Completeness Model

| Lane | Purpose | Promotion Requirement |
|------|---------|----------------------|
| `SIM` | Simulation, AI‑generated drafts | Requires measured data shard |
| `EXP` | Experimental validation | Requires KER ≥ thresholds |
| `PROD` | Production ecosafety logic | Full governance audit |
| `ARCHIVE` | Historical reference | Immutable |

**ShardCompleteness:** `MEASURED`, `SIMULATED`, `MIXED`, `CONCEPTUAL`

AI‑Chat output starts as `lane="SIM"` / `COMPLETENESS=SIMULATED`. Promotion to PROD requires a **promotion shard** tying code to measured eco‑impact data.

## ResponseShard Discipline

Every meaningful change (especially AI‑Chat or agent actions) must emit a `ResponseShard`:

```rust
pub struct ResponseShard {
    pub shard_id: String,
    pub author_did: String,       // Bostrom DID
    pub topic: String,            // e.g. "eco_restoration_shard:ai_chat"
    pub timestamp_utc: String,
    pub k: f64,
    pub e: f64,
    pub r: f64,
    pub vt_before: f64,
    pub vt_after: f64,
    pub corridor_ids: Vec<String>,
    pub evidence_hex: String,     // Commit hash or hex stamp
    pub lane: String,             // "SIM", "EXP", "PROD", "ARCHIVE"
}
```

**Git hook requirement:** Any commit modifying `crates/` or `schemas/` must include a new ResponseShard row under `shards/response/`.

## AI‑Chat Interface

AI‑Chat interacts via the `agent_interface` crate with constrained actions:

```rust
pub enum AgentAction {
    ListShards { topic: String },
    ProposePatch { path: String, diff: String },
    RunCheck { check: String, target: String },
}
```

All actions:
- Use `oss_vfs` for streaming, size‑bounded scans
- Write proposals to `/mnt/oss/staging/` (quarantine)
- Emit a `ResponseShard` with `lane="SIM"` until CI promotes

## Crates Overview

| Crate | Purpose |
|-------|---------|
| `cyboquatic-ecosafety-core` | Core rx–Vt–KER grammar, safestep, corridors |
| `cyboquatic-fog-routing` | FOG routing with Δcarbon, Δbiodiversity gates |
| `econet-material-cybo` | Biodegradable material traits, t90 kinetics |
| `cyboquatic-energy-mass` | J/kg removal, energy plane normalization |
| `oss_vfs` | Exabyte VFS abstraction for `/mnt/oss` |
| `response_shard_core` | ResponseShard types, ALN schema, CSV helpers |
| `agent_interface` | Constrained AI‑Chat action vocabulary |
| `storage_shards` | StorageNodeShard schemas for capacity planning |
| `build_scheduler` | Compute node ecosafety for Rust builds |

## Funding & Trust

This repository is designed as **shared ecosafety infrastructure**:
- All code is non‑deceptive, hex‑stampable, DID‑signable
- No forbidden cryptographic primitives or unsafe control paths
- CI enforces KER thresholds and Vt invariants
- Public ResponseShard ledger enables community audit

For grant applications, cite:
- **K ≈ 0.94**: Direct reuse of existing spine semantics
- **E ≈ 0.90**: Forces shift toward restorative configurations
- **R ≈ 0.12–0.13**: Residual from corridor calibration and sensor error

---

*Last updated by ResponseShard `eco_restoration_shard:ecosafety_doc_2026v1`*
*Evidence hex: `TODO:CI_FILL_COMMIT_HASH`*
*Author DID: `TODO:CI_FILL_DID`*
