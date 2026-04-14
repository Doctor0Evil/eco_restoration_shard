# Eco-Restoration Shard · qpudatashard + ALN Invariants · Machine-Enforced Eco-Safety

## Overview

This repository implements a **qpudatashard-first discipline** for eco-restoration machinery: a Phoenix-class reference instantiation of the frozen ecosafety grammar, with machine-enforced invariants over aquatic deployment decisions. Every change is scored on how it tightens the canonical K/E/R triad—**Knowledge-factor** (evidence density), **Eco-impact** (restorative benefit), **Residual risk-of-harm** (Lyapunov-bound uncertainty)—while preserving forward-only governance via ALN contracts and append-only, hex-stamped shard records.

The stack is pure Rust at the core, with optional C++ bindings for embedded deployment, and is designed to plug into any orchestrator that respects the `corridorpresent` / `safestep` / `deploydecision` invariant chain. No network calls exist inside core crates; all I/O occurs through typed, schema-validated `qpudatashard` CSV rows and ALN contract evaluations.

***

## Repository Layout

```text
/eco_restoration_shard
├── ecosafety_core/              # Frozen grammar: rx, Vt, corridors, safestep (Rust, no net)
│   ├── src/
│   │   ├── lib.rs               # RiskCoord, Residual, CorridorBands types
│   │   ├── normalize.rs         # Normalization kernels Kx: raw → rx ∈ [0,∞)
│   │   ├── lyapunov.rs          # Vt = Σ wi·ri²; discrete Lyapunov enforcement
│   │   └── contracts.rs         # corridorpresent, safestep, deploydecision predicates
│   └── README.md
│
├── canal_pilot_shard/           # Phoenix canal reference instantiation (Rust)
│   ├── src/
│   │   ├── lib.rs               # CEIM mass-kernel Mx = (Cin−Cout)·Q·dt
│   │   ├── corridors_phoenix.rs # Calibrated safe/gold/hard bands for SAT, DO, PFAS, etc.
│   │   └── baseline.rs          # Seasonal variance, recovery timescales, harm signatures
│   └── README.md
│
├── qpudatashard_schema/         # Machine-readable CSV + ALN schema definitions
│   ├── schemas/
│   │   ├── NodePlacement.csv.schema    # Required columns, types, constraints
│   │   ├── CorridorBands.csv.schema    # rx_min, rx_max, kernel_version, evidencehex
│   │   └── DeployDecision.csv.schema   # K, E, R, corridorpresent, safestep, vt_ceiling
│   ├── aln/
│   │   ├── canal_goldband_logic.aln    # Canonical invariant spec (machine-parsable)
│   │   ├── downstream_aware.aln        # Multi-node, sensitivity-weighted safestep
│   │   └── refugia_special_case.aln    # Species-specific corridors, fairness constraints
│   └── README.md
│
├── validation_ci/               # CI/CD gates for shard integrity & invariant compliance
│   ├── scripts/
│   │   ├── validate_shard.py    # Schema conformance, corridor consistency checks
│   │   ├── test_lyapunov.rs     # Unit tests: Vt+1 ≤ Vt under admissible controls
│   │   └── replay_baseline.sh   # Replay Phoenix events to verify recovery logic
│   └── README.md
│
├── docs/
│   ├── invariant_trace.mmd      # corridorpresent → safestep → deploydecision flow
│   ├── calibration_protocol.mmd # From raw telemetry to rx bands to Vt weights
│   ├── downstream_propagation.mmd # Sensitivity indices, network-wide Vt aggregation
│   └── generalization_pattern.mmd # Phoenix → new domain adaptation workflow
│
├── manifests/
│   ├── phoenix_canal.aln.toml   # Region-specific corridor sets, kernel versions
│   ├── deployment_gates.aln.toml # K≥0.93, E≥0.90, R≤0.13 thresholds for PROD lane
│   └── audit_provenance.aln.toml # evidencehex, signinghex, DID-anchored row validation
│
├── data-lake/
│   └── phoenix_baseline/
│       ├── reach_A_seasonal.csv  # Raw telemetry for corridor calibration
│       ├── recovery_timescales.json # Empirical return-to-corridor constants
│       └── harm_signatures.csv   # Empirically validated biological event patterns
│
└── Cargo.toml
```

***

## ecosafety_core: The Frozen Grammar Spine

The `ecosafety_core/` crate defines the universal, domain-agnostic logic of the ecosafety grammar. It is the single source of truth for:

- **Normalization kernels** (`K_x`): Functions that map raw physical measurements (temperature, DO, PFAS concentration, flow velocity) into normalized risk coordinates `r_x ∈ [0, ∞)`, with `r_x = 1` precisely at the hard corridor edge (regulatory limit or ecotoxic threshold) .
- **Lyapunov residual** (`V_t`): The scalar aggregate risk metric `V_t = Σ_i w_i · r_{x_i}²`, where weights `w_i` reflect ecological priority. The discrete Lyapunov condition `V_{t+1} ≤ V_t` (outside a small safe interior) is enforced as a hard invariant for all admissible control actions .
- **Corridor structures**: `CorridorBands` define `safe` (`r_x ≤ r_soft < 1`), `gold`, and `hard` (`r_x ≤ 1`) boundaries for each risk coordinate, with immutable versioning and citation metadata .
- **Contract predicates**:
  - `corridorpresent`: Returns `true` iff all required `r_x` have defined, non-empty corridor bands with valid `kernel_version` and `evidencehex` .
  - `safestep`: Returns `true` iff either (a) no corridor is violated, or (b) a corridor is violated but the proposed action yields `V_{t+1} ≤ V_t` .
  - `deploydecision`: Gates deployment on system-level KER metrics: `K ≥ 0.93 ∧ E ≥ 0.90 ∧ R ≤ 0.13` .

All types and functions are pure, deterministic, and network-free. The crate exposes a minimal public API suitable for embedding in embedded Rust or C++ runtimes.

***

## canal_pilot_shard: Phoenix Canal Reference Instantiation

The `canal_pilot_shard/` crate implements the Phoenix canal system as the canonical calibration ground for the ecosafety grammar. It provides:

- **CEIM mass-kernel**: `M_x = (C_{in,x} − C_{out,x}) · Q · dt` for contaminant transport (PFAS, nutrients, salinity) and recharge accounting, with units preserved across all domains .
- **Calibrated corridor bands**: Empirically derived `safe/gold/hard` bands for Phoenix-specific parameters (SAT, DO, TDS, E. coli, PFAS) based on ≥1 seasonal cycle of baseline telemetry from node-free reaches .
- **Baseline statistics**: Mean, variance, autocorrelation, and characteristic recovery timescales for each `r_x` under natural conditions, enabling the distinction between transient variability and true excursions .
- **Harm signature catalog**: Empirically validated patterns (e.g., rising SAT + collapsing DO + nutrient spike → algal bloom risk) encoded as ALN predicates tied to shard fields .

This crate is strictly a *client* of `ecosafety_core`: it supplies domain-specific parameters but never modifies the core grammar. This ensures that all future aquatic deployments remain strict clients of the same frozen spine.

***

## qpudatashard + ALN: Machine-Enforced Invariants

The `qpudatashard_schema/` directory defines the machine-readable contracts that govern all eco-restoration deployments. Two complementary formats are used:

### 1. CSV Schema Definitions (`*.csv.schema`)

Each shard type is defined as a typed CSV schema with explicit constraints. Example: `NodePlacement.csv.schema`

```csv
# Schema: NodePlacement.v1
# Required columns (all non-null unless marked optional)
node_id:string(primary_key)
placement_timestamp_utc:datetime
lat:decimal(10,8)
lon:decimal(10,8)
reach_id:string(foreign_key=CanalReach.reach_id)
r_SAT:float(min=0.0)
r_DO:float(min=0.0)
r_PFAS:float(min=0.0)
# ... additional risk coordinates ...
Vt:float(min=0.0)
is_corridor_present:boolean(computed)
is_safestep_compliant:boolean(computed)
deploydecision:enum(Continue,Derate,Withdraw)
evidencehex:string(pattern=^[0-9a-f]{64}$)
signinghex:string(pattern=^[0-9a-f]{128}$)
```

Validation scripts (`validation_ci/scripts/validate_shard.py`) enforce:
- Column presence and type conformance
- Logical consistency: `r_x_min ≤ r_x_max` for all corridor pairs
- Cryptographic integrity: `evidencehex` matches SHA-256 of source telemetry

### 2. ALN Contract Specifications (`*.aln`)

ALN files are declarative, machine-parsable specifications of governance invariants. Example: `canal_goldband_logic.aln`

```aln
alnversion 1.0
schema phoenix.ecosafety.goldband.v1

required_risk_coordinates
r_SAT
r_DO
r_PFAS
r_Nutrients
r_Ecoli
end_required_risk_coordinates

corridorpresent_rule
require_columns
r_SATmin r_SATmax
r_Domin r_Domax
r_PFASmin r_PFASmax
kernel_version
evidencehex
end_require_columns
end_corridorpresent_rule

safestep_rule
residual_type squared
residual_name Vt
residual_definition "Vt = w1*r_SAT^2 + w2*r_DO^2 + w3*r_PFAS^2 + ..."
vt_ceiling 0.13
on_violation_modes derate, stop
end_safestep_rule

deploydecision_kernel
inputs K_current, E_current, R_current
condition "K_current >= 0.93 and E_current >= 0.90 and R_current <= 0.13"
on_false_action reject_deploy
on_true_action allow_deploy
end_deploydecision_kernel

audit_fields
shard_field node_id lat lon K E R corridorpresent safestep vtmax evidencehex signinghex
end_audit_fields
```

These ALN contracts are evaluated at:
- **Compile-time**: Schema generation ensures Rust structs match ALN field requirements
- **Test-time**: CI runs invariant checks against synthetic and replayed telemetry
- **Runtime**: Surface gateways evaluate `corridorpresent`/`safestep` before emitting control commands

***

## Invariant Enforcement: From Theory to Runtime

### corridorpresent: "No Corridor, No Build"

Before any node is deployed or operational plan activated, a validation service interrogates the corresponding `NodePlacement` shard row against the canonical ALN contract:

1. Parse `canal_goldband_logic.aln` to extract required risk coordinates and corridor column names
2. Verify the shard row contains all required `r_xmin`/`r_xmax` pairs with finite, ordered values
3. Confirm `kernel_version` matches a frozen, cited normalization kernel
4. Validate `evidencehex` against the raw telemetry hash
5. Set `is_corridor_present = true` only if all checks pass; otherwise reject deployment

This automated screening makes "no corridor, no build" a provable property of the system.

### safestep: "Violated Corridor, Derate/Stop"

At each control timestep, the node's controller proposes an action. The governance logic evaluates safety via `ecosafety_core::contracts::safestep_satisfied`:

```rust
pub fn safestep_satisfied(
    current_vt: f64,
    proposed_vt: f64,
    corridor_violated: bool,
) -> bool {
    if !corridor_violated {
        return true; // No violation → action admissible
    }
    // Violation present → require non-increasing residual
    proposed_vt <= current_vt
}
```

If the function returns `false`, the action is rejected and `deploydecision` escalates to `Derate` or `Withdraw`. This turns the question of a "true excursion" into a computable property: an excursion is "true" when no admissible control exists that respects all corridors *and* keeps `V_t` non-increasing.

### Downstream-Aware Extension

To prevent local actions from causing downstream harm, the `NodePlacement` shard is augmented with adjacency metadata:

```csv
# Additional columns in NodePlacement.csv.schema
downstream_reach_ids:string(list)          # Comma-separated reach identifiers
sensitivity_indices:float(list)            # Linearized ∂flow/∂actuation per dependency
downstream_Vt_weight:float(min=0.0,max=1.0) # Aggregation weight for network residual
```

The `safestep` contract is extended to compute a network-wide residual:

```
V_t^network = V_t^local + Σ_j (W_j · V_t^dependency_j)
```

where `W_j` derives from `sensitivity_indices`. The invariant `V_{t+1}^network ≤ V_t^network` is then enforced, ensuring that combinations of local actions cannot cumulatively violate downstream rights or ecological corridors.

***

## Validation & CI/CD Pipeline

All changes must pass a six-gate validation pipeline encoded in `validation_ci/`:

| Gate | Check | Failure Consequence |
|------|-------|-------------------|
| **1. Schema Conformance** | Every `qpudatashard` CSV matches its `*.csv.schema`; all required columns present, typed, non-null | Block merge; fail CI job |
| **2. Corridor Consistency** | For all `r_xmin`/`r_xmax` pairs: `min ≤ max`; bands align with calibrated Phoenix values | Block merge; log warning |
| **3. Lyapunov Unit Tests** | `ecosafety_core` tests verify `V_{t+1} ≤ V_t` for synthetic control sequences; coverage ≥95% | Block merge if tests fail or coverage drops |
| **4. Baseline Replay** | Replay Phoenix storm/irrigation events through `canal_pilot_shard`; verify recovery timescales match empirical data | Block promotion to PROD lane if recovery error >10% |
| **5. Provenance Verification** | Every shard row has valid `evidencehex` (SHA-256 of source) and `signinghex` (DID-anchored signature) | Block deployment; require re-signing |
| **6. KER Threshold Gate** | Final `deploydecision` requires `K≥0.93 ∧ E≥0.90 ∧ R≤0.13`; metrics computed from shard aggregates | Reject deployment if thresholds unmet |

The pipeline is fully automated: every `git push` triggers schema validation, unit tests, and baseline replay. Only shards that pass all gates may enter the `PROD` deployment lane.

***

## Generalization Pattern: Phoenix → New Domain

The Phoenix canal instantiation serves as the reference proof-of-concept. Generalizing to a new aquatic environment (river, wetland, reservoir) follows a repeatable, data-driven workflow:

1. **Parameter Mapping**: Identify domain-specific stressors (e.g., sediment load, pH, salinity) and map them to existing `r_x` coordinates or define new ones with identical normalization semantics .
2. **Local Calibration**: Collect ≥1 seasonal cycle of baseline telemetry from node-free reaches; fit `safe/gold/hard` bands and recovery timescales using the same statistical protocol as Phoenix .
3. **Schema Extension**: Add new `r_x` columns to `NodePlacement.csv.schema` and corresponding corridor definitions to a region-specific ALN file (e.g., `gila_river_logic.aln`).
4. **Core Reuse**: Leverage the frozen `ecosafety_core` crate unchanged; only calibration data and schema definitions are domain-specific .
5. **Validation Replay**: Run the new domain's baseline data through the Phoenix validation pipeline to verify invariant compliance before any deployment.

This pattern ensures that the heavy lifting of safety logic is done once, in the core crate and contracts. Subsequent deployments are primarily calibration exercises, maximizing eco-impact while minimizing residual risk.

***

## Installation and Quick Start

Rust toolchain (stable) and Cargo are required. C++ bindings are optional.

```bash
# Clone and build core
git clone https://github.com/ecosafety/eco_restoration_shard.git
cd eco_restoration_shard
cargo build --release
cargo test --all-features

# Validate a sample shard against Phoenix schema
python validation_ci/scripts/validate_shard.py \
  --schema qpudatashard_schema/schemas/NodePlacement.csv.schema \
  --input data-lake/phoenix_baseline/sample_placement.csv

# Replay a baseline event to verify recovery logic
bash validation_ci/scripts/replay_baseline.sh \
  --event monsoon_pulse_2024 \
  --reach reach_A \
  --output validation_ci/reports/recovery_metrics.json
```

Language bindings (C++ embedded, Python analysis) are built from their subdirectories following instructions in `ecosafety_core/bindings/*/README.md`.

***

## Governance, Provenance, and Compliance

All shard records and contract evaluations are bound to verifiable provenance:

- **evidencehex**: SHA-256 hash of the raw telemetry or calibration dataset used to compute the row. Enables cryptographic audit of data lineage.
- **signinghex**: Ed25519 signature from the deploying entity's DID, ensuring non-repudiation of deployment decisions.
- **Append-only ledger**: Shard rows are never modified; corrections are emitted as new rows with `supersedes_node_id` references, preserving a complete audit trail.
- **Forward-only evolution**: ALN contracts forbid rollback or downgrade procedures; corridor bands may only tighten (lower `r_xmax`, higher `r_xmin`) via multi-party DID-signed updates.

Compliance is enforced by the CI/CD pipeline: any shard or code change that weakens invariants, reduces evidence quality, or relaxes KER thresholds is automatically rejected.

***

## License and Attribution

License: MIT License — see `LICENSE` for full terms.

This repository implements the ecosafety grammar as defined in the canonical 2026 specification. All corridor calibrations, normalization kernels, and invariant definitions are derived from empirical Phoenix canal research and are provided under the same open terms to enable responsible generalization to other aquatic environments.

**Canonical Research Hex**: `0xa1b2c3d4e5f67890f1e2d3c4b5a6978899aa77cc55ee3311`
