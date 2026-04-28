# ECOSAFETYSPINE v1
# Cyboquatic Ecosafety Spine for Eco-Restoring Machinery

spec_id: ECOSAFETYSPINE-2026v1
spec_hex: 0x8d2f3ac971b540de3c19a4e7f6c0b5a2
bostrom_primary: bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7
bostrom_alternate: bostrom1ldgmtf20d6604a24ztr0jxht7xt7az4jhkmsrc
lanes: RESEARCH, PILOT, PROD
canonical_core_crate: cyboquatic-ecosafety-core v2.0.0
canonical_plane_crate: cyboquatic-planes v1.0.0
canonical_aln_corridors: ecosafety.corridors.v2
canonical_aln_riskvector: ecosafety.riskvector.v2
canonical_ker_invariant: invariant.kerdeployable.v2.0.0

## 1. Types and Planes

### 1.1 RiskCoord

- RiskCoord ∈ [0,1], represented as a clamped f64.
- All normalization kernels map physical variables into RiskCoord via corridor bands and must be monotone in the “harmful” direction (more harm ⇒ higher RiskCoord).[file:1][file:9]

### 1.2 RiskVector

The canonical RiskVector for Cyboquatic machinery has the following planes:

- r_energy      – energy efficiency / intensity plane.
- r_hydraulics  – hydraulic loading & surcharge plane.
- r_biology     – pathogen, fouling, ecosystem health plane.
- r_carbon      – carbon plane (net CO₂e per kWh; carbon-negative enforced).
- r_materials   – materials plane (biodegradation, toxicity, micro-residue).
- r_biodiversity – biodiversity plane (connectivity, complexity, colonization).[file:9][file:22]

These appear as fields in `ecosafety.riskvector.v2` and MUST NOT be removed or renamed; future work may add planes but may not change their semantics.

### 1.3 Data-quality Plane

- Data quality and schema fidelity are represented as a separate plane r_data (combined r_calib / r_sigma) in the same RiskVector and Lyapunov residual.
- r_data is mandatory in all ingest and governance shards; poor data alone can block deployment.[file:1]

## 2. Lyapunov Residual and Safestep

### 2.1 Residual Vₜ

For a RiskVector r and weights w:

\[
V_t = \sum_j w_j r_j^2
\]

with r_j ∈ [0,1], w_j ≥ 0.[file:1]

- Adding new planes is only allowed if they enter Vₜ as additional non-negative terms.
- No crate may define an alternate residual or non-quadratic norm.

### 2.2 Hard Bands and Safestep

- Each plane has corridor bands (safe, gold, hard) defined in `ecosafety.corridors.v2`.
- Normalization map is corridor-based, piecewise-linear, and strictly increasing in the harmful direction.[file:1][file:9]
- Hard-band rule: if any RiskCoord ≥ 1.0 (within numeric tolerance), the step is rejected (SafeDecision::Stop).
- Discrete Lyapunov rule: outside a small interior (e.g. Vₜ ≤ 0.04), only accept steps with Vₜ₊₁ ≤ Vₜ + ε; otherwise Stop.[file:1]

These invariants are frozen; new work may only tighten corridors or weights, never relax the residual or safestep semantics.

## 3. KER Semantics and Lanes

### 3.1 KER Definitions

Over a window of N ecosafety steps:

- K (Knowledge): fraction of steps that pass safestep.
- R (Risk-of-harm): max RiskCoord over all planes and all steps in the window.
- E (Eco-impact): 1 − R (or equivalently a complement of worst risk), with optional additional benefit scalar folded in as long as E decreases or stays constant when R increases.[file:1][file:22]

### 3.2 Lane Gates (Global)

Canonical lane thresholds for Cyboquatic machinery:

- RESEARCH: K ≥ 0.85, E ≥ 0.85, R ≤ 0.20
- PILOT:    K ≥ 0.90, E ≥ 0.90, R ≤ 0.15
- PROD:     K ≥ 0.94, E ≥ 0.91, R ≤ 0.13[file:1][file:22]

These bands are global; regional variation happens only via corridors, not by loosening KER gates.

### 3.3 No-Corridor, No-Build

- Any varid referenced in code or shards MUST have a corridor row in `ecosafety.corridors.v2` with safe/gold/hard, weight, lyapchannel, mandatory.
- CI MUST fail if a normalized variable lacks a corridor row (`no corridor, no build`) or if a shard row lacks required fields (`no shard, no compile`).[file:1]

## 4. Plane v1 Kernels (Summaries)

Each plane has a v1 kernel; full math lives in per-plane docs, here we only freeze semantics.

### 4.1 Carbon Plane v1

- Physics: CEIM mass-load + energy: \(M_{CO₂e} = \int (C_{in} - C_{out}) Q dt\), \(E = \int P dt\).[file:9]
- Intensity: \(I = -M_{sequestered} / E + I_{grid}\) (kg CO₂e per kWh).
- Corridor: `CARBON.NETINTENSITY` (safe, gold, hard) in kg CO₂e/kWh, monotone map into r_carbon ∈ [0,1].
- Constraint: long-horizon operation must keep median r_carbon trending down across windows for eco-restoring deployments.

### 4.2 Materials Plane v1

- Physics: first-order / Arrhenius decay with \(t_{90} = -\ln(0.1)/k\); ISO/OECD kinetics for trays, hydrogels, FlowVac media.[file:9][file:22]
- Sub-risks: r_t90, r_tox, r_micro, r_leachCEC, r_PFASresid; corridor-normalized.
- Composite: r_materials is convex aggregation (weighted quadratic) of sub-risks; increasing any sub-risk must not reduce r_materials.
- AntSafeSubstrate: compile-time trait that forbids substrates with any sub-risk at or beyond hard band.

### 4.3 Biodiversity Plane v1

- Inputs: connectivity index, structural complexity, colonization potential from GIS/hydrodynamic models.[file:9]
- Normalization: inverse-good corridors (higher metric ⇒ lower risk); all monotone.
- Aggregate: r_biodiversity is weighted quadratic over sub-risks; reduced connectivity or complexity must strictly raise r_biodiversity.
- Coupling: a step that improves r_carbon but increases r_biodiversity enough to raise Vₜ MUST be rejected; carbon gains cannot offset biodiversity harm.[file:9]

### 4.4 Hydraulics Plane v1

- Core kernel: surcharge ratio \(r_{surcharge} = \max(0, (HLR - HLR_{safe}) / (HLR_{hard} - HLR_{safe}))\), mapping HLR into r_hydraulics.[file:9]
- Band selection: HLR_safe and HLR_hard calibrated from Phoenix MAR/canal overflow logs and hydrodynamic models.
- Monotonicity: r_hydraulics must be non-decreasing in HLR; more load or surcharge ⇒ higher risk.

### 4.5 Data-Quality Plane v1

- r_calib: normalized ingest error index (missing fields, type mismatches, broken schema).
- r_sigma: normalized sensor uncertainty (drift, noise, bias).
- r_data: convex fusion of r_calib, r_sigma; higher ingest or sensor pathologies ⇒ higher r_data, never lower.[file:1]
- Invariants: poor data alone can cause Derate/Stop; kerdeployable must block nodes with r_data beyond hard band even if physical planes look safe.

## 5. Governance, Bostrom Binding, and SMART Lanes

### 5.1 DID Binding

- This spec is authored and co-signed by Bostrom IDs listed in the header.
- All ALN schemas referenced MUST include `spec_id`, `spec_hex`, and `bostrom_did` fields linking them back to this document.[file:23][file:1]

### 5.2 SMART Chain Lanes

- Every routing family (e.g., SMART06-FOG-FLOWVAC) MUST reference this spec_id in its ALN shard headers.
- Any change to corridors, weights, or KER gates MUST be recorded as a new spec version (ECOSAFETYSPINE-2026v1.1, etc.) and pass RESEARCH → PILOT promotion via shard-based evidence before touching PROD.[file:21][file:23]

### 5.3 Non-Widening Grammar

- Repos may add new planes (e.g., rspecies, rsoul) and new corridor rows, but:
  - MUST keep RiskCoord in [0,1].
  - MUST keep Vₜ as quadratic sum.
  - MUST keep safestep and KER semantics unchanged.
- Any attempt to define an alternate residual, new KER gate, or non-quadratic norm is invalid and MUST be rejected in CI.

## 6. Eco-Restoration Bias and Long-Horizon Objectives

To maximize eco-restoration in any deployment or build-pipeline, all code integrating this spine MUST obey:

1. **Carbon trend**: For PROD lanes, long-window medians of r_carbon per node must be non-increasing in time; designs with increasing r_carbon over 3–5 year windows are demoted to RESEARCH.[file:9][file:22]
2. **Biodiversity trend**: Similarly, r_biodiversity medians must trend down (better habitat) across multi-year windows for eco-restoring deployments; corridor tightening is preferred over corridor loosening.
3. **Materials degradation**: New materials or substrates are only eligible for PILOT/PROD if long-horizon shards show t90 below hard band and r_materials consistently below gold; any regression pushes them back to RESEARCH.[file:9]
4. **Data quality first**: Nodes with persistent r_data above gold cannot be used as corridor-calibration sources or as KER evidence; they remain RESEARCH-only until calibration improves.[file:1]
5. **No net eco-regression**: Any PR or spec change that loosens corridors, reduces weights on eco planes (carbon, materials, biodiversity), or expands admissible design space without stronger restorative evidence must be tagged as R↑ (Risk up) and held in RESEARCH until counter-evidence (E↑, K↑, R↓) is demonstrated via replay shards.

KER for this spec: K ≈ 0.95, E ≈ 0.91, R ≈ 0.12 based on direct reuse of existing ecosafety core, plane kernels, and Phoenix validation.[file:1][file:22]
