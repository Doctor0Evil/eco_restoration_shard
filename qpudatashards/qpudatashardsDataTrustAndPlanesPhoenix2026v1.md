# qpudatashards data‑trust and extended‑plane structuring (Phoenix 2026v1)

This document captures the qpudatashard‑visible structuring changes for data‑trust planes (rcalib, rsigma) and the extended ecological planes (rcarbon, rtox, rbiodiversity), in the grammar of the existing rx–Vt–KER spine.

## 1. File layout and discoverability

To keep these shards maximally discoverable and consistent with existing EcoNet and cyboquatic patterns, use:

- Destination directory
  - `qpudatashards/` at the top level of the repo that already carries Phoenix ecosafety particles.
- Primary filename for the data‑trust and extended‑planes particle
  - `qpudatashards/particlesCyboquaticDataTrustPhoenix2026v1.csv`
- Primary ALN manifest for corridors, KER, and invariants
  - `qpudatashards/DataTrustAndPlanesPhoenix2026v1.aln`

These sit alongside existing planning‑grade shards such as `PlanningSafetySecurityAirWater2026v1.csv` and Phoenix ecosafety particles, so that discovery by pathname and by pattern (`Phoenix2026v1`) is straightforward.

## 2. CSV shard structure: particlesCyboquaticDataTrustPhoenix2026v1.csv

The CSV shard carries per‑window KER and plane coordinates for Phoenix nodes, in a form directly consumable by the ecosafety core and ALN.

**Filename**

- `qpudatashards/particlesCyboquaticDataTrustPhoenix2026v1.csv`

**Header**

```text
nodeid,medium,region,twindowstart,twindowend,rcalib,rsigma,rtox,rbiodiversity,rcarbon,Vt_full,K_raw,E_raw,K_adj,E_adj,R,evidencehex
```

**Column semantics**

- `nodeid`
  Canonical node identifier (e.g., MAR‑PHX‑01, CANAL‑GILA‑07, AG‑PHX‑01).
- `medium`
  Qualitative tag for the physical medium (`water`, `air`, `soil`, etc.).
- `region`
  Human‑readable georegion label (e.g., `Phoenix-AZ`).
- `twindowstart`, `twindowend`
  ISO timestamps delimiting the closed ingest / evaluation window.
- `rcalib`
  Normalized ingest data‑trust coordinate in [0,1]. 0 means deeply safe ingest; 1 denotes chronic ingest failure bands.
- `rsigma`
  Composite sensor‑uncertainty coordinate in [0,1], aggregating drift, noise, bias, and loss.
- `rtox`
  Materials / toxicity risk coordinate in [0,1], derived from plane‑specific physics kernels (decay, ecotox, micro‑residue, leachate, PFAS‑like residuals) via a weighted norm.
- `rbiodiversity`
  Biodiversity risk coordinate in [0,1], constructed as an “inverse‑good” fusion of connectivity, structural complexity, and colonization metrics.
- `rcarbon`
  Net carbon‑intensity risk coordinate in [0,1], derived from CEIM mass kernels and net intensity (including sequestration and grid intensity contributions).
- `Vt_full`
  Extended Lyapunov residual for the window, including all physical planes plus rcalib and rsigma.
- `K_raw`
  Window‑level knowledge score from the base safestep rule (fraction of timesteps where safestep holds).
- `E_raw`
  Window‑level eco‑impact score computed as `1 - R_raw` from raw coordinates.
- `K_adj`
  Knowledge score down‑scaled by combined data‑trust (`Dcombined`).
- `E_adj`
  Eco‑impact score down‑scaled by combined data‑trust, ensuring that poor data quality cannot inflate credited benefit.
- `R`
  Window‑level residual risk, defined as the maximum RiskCoord across all planes, including rcalib and rsigma.
- `evidencehex`
  Hex string tying the record back to the underlying ingest shards, replay profile, and ALN proofs.

**Sample rows**

```text
MAR-PHX-01,water,Phoenix-AZ,2026-01-20T00:00:00Z,2026-01-21T00:00:00Z,0.04,0.07,0.22,0.18,0.11,0.062,0.93,0.91,0.89,0.88,0.12,a1b2c3d4e5f67890

CANAL-GILA-07,water,Gila-AZ,2026-01-20T00:00:00Z,2026-01-21T00:00:00Z,0.09,0.12,0.27,0.21,0.15,0.085,0.92,0.90,0.86,0.84,0.13,1122334455667788

AG-PHX-01,air,Phoenix-AZ,2026-01-20T00:00:00Z,2026-01-21T00:00:00Z,0.06,0.08,0.19,0.24,0.17,0.073,0.91,0.89,0.86,0.84,0.14,c5d6e7f8a9b0c1d2
```

This shard is intentionally narrow: it carries just enough structure to support KER evaluation, replay CI, and governance decisions, while deferring detailed physics kernels to upstream shards.

## 3. ALN manifest: DataTrustAndPlanesPhoenix2026v1.aln

The ALN manifest binds the CSV shard into the existing ecosafety grammar: it defines corridors for the new planes, attaches Lyapunov weights, enforces “no corridor, no build,” and wires in non‑compensability and data‑trust gates.

**Filename**

- `qpudatashards/DataTrustAndPlanesPhoenix2026v1.aln`

### 3.1 Corridors table

The `corridors` table extends `ecosafety.corridors.v2` with rows for the data‑trust and ecological planes.

```aln
particle DataTrustAndPlanesPhoenix2026v1

meta
  region      Phoenix-AZ
  version     2026v1
  description Data-trust and extended ecological planes for Phoenix qpudatashards


table corridors
  -- Data-trust planes
  row varid rcalib
      units       dimensionless
      safemin     0.0
      safemax     0.00
      goldmin     0.00
      goldmax     0.30
      hardmin     0.30
      hardmax     1.00
      rgold       0.50
      lyapweight  1.0
      mandatory   true
      channel     dataquality

  row varid rsigma
      units       dimensionless
      safemin     0.0
      safemax     0.10
      goldmin     0.10
      goldmax     0.30
      hardmin     0.30
      hardmax     1.00
      rgold       0.50
      lyapweight  1.0
      mandatory   true
      channel     dataquality

  -- Extended ecological planes
  row varid rcarbon
      units       dimensionless
      safemin     -1.00   -- net sequestration safe band
      safemax     0.00
      goldmin     0.00
      goldmax     0.20
      hardmin     0.20
      hardmax     1.00
      rgold       0.50
      lyapweight  1.0
      mandatory   true
      channel     carbon

  row varid rtox
      units       dimensionless
      safemin     0.0
      safemax     0.05
      goldmin     0.05
      goldmax     0.30
      hardmin     0.30
      hardmax     1.00
      rgold       0.50
      lyapweight  1.0
      mandatory   true
      channel     materials

  row varid rbiodiversity
      units       dimensionless
      safemin     0.0     -- low risk when ecosystems are healthy
      safemax     0.05
      goldmin     0.05
      goldmax     0.30
      hardmin     0.30
      hardmax     1.00
      rgold       0.50
      lyapweight  1.0
      mandatory   true
      channel     biodiversity
```

### 3.2 Risk vector and KER window schema

The `riskvector` and `kerwindow` tables make the extended RiskVector explicit and keep KER semantics aligned with the frozen ecosafety core.

```aln
table riskvector
  -- Physical planes (existing)
  col r_hydraulics   double
  col r_energy       double
  col r_materials    double
  col r_biology      double

  -- New ecological planes
  col r_carbon       double  maps varid rcarbon
  col r_tox          double  maps varid rtox
  col r_biodiv       double  maps varid rbiodiversity

  -- Data-trust planes
  col r_calib        double  maps varid rcalib
  col r_sigma        double  maps varid rsigma


table kerwindow
  col nodeid         text
  col twindowstart   timestamp
  col twindowend     timestamp

  col Vt_full        double
  col K_raw          double
  col E_raw          double
  col R_raw          double

  col K_adj          double
  col E_adj          double

  col D_sensor       double
  col D_data         double
  col D_combined     double

  col DeployDecision text
  col evidencehex    text
  col signinghex     text
```

### 3.3 Invariants: corridor presence and data‑trust gates

The invariants in this particle enforce that all risk coordinates have corridors, and that data‑trust planes act as hard gates.

```aln
invariant NoCorridorNoBuild
  kind governance
  applies_to kerwindow
  for_all rows w in kerwindow
    require corridor_present(rcalib)
    require corridor_present(rsigma)
    require corridor_present(rcarbon)
    require corridor_present(rtox)
    require corridor_present(rbiodiversity)


invariant BlockedByCalib
  kind kerdeployable
  applies_to kerwindow
  for_all rows w in kerwindow
    let rc = w.r_calib in
    if rc > corridors.rcalib.hardmin then
      require w.DeployDecision != "PROD"
    end


invariant BlockedBySigma
  kind kerdeployable
  applies_to kerwindow
  for_all rows w in kerwindow
    let rs = w.r_sigma in
    if rs > corridors.rsigma.hardmin then
      require w.DeployDecision != "PROD"
    end
```

### 3.4 Invariants: non‑compensability above gold bands

Non‑compensability is encoded as a family of typed invariants, one per critical plane.

```aln
invariant NoOffsetAboveGold_carbon
  kind lyapunov
  applies_to kerwindow
  for_all transitions (w_t, w_t1) over same nodeid
    let r_t   = w_t.r_carbon in
    let r_t1  = w_t1.r_carbon in
    let gold  = corridors.rcarbon.goldmax in
    if r_t < gold && r_t1 >= gold then
      require w_t1.DeployDecision != "PROD"
      require w_t1.E_adj <= w_t.E_adj
      require w_t1.R_raw >= w_t.R_raw
    end


invariant NoOffsetAboveGold_tox
  kind lyapunov
  applies_to kerwindow
  for_all transitions (w_t, w_t1) over same nodeid
    let r_t   = w_t.r_tox in
    let r_t1  = w_t1.r_tox in
    let gold  = corridors.rtox.goldmax in
    if r_t < gold && r_t1 >= gold then
      require w_t1.DeployDecision != "PROD"
      require w_t1.E_adj <= w_t.E_adj
      require w_t1.R_raw >= w_t.R_raw
    end


invariant NoOffsetAboveGold_biodiv
  kind lyapunov
  applies_to kerwindow
  for_all transitions (w_t, w_t1) over same nodeid
    let r_t   = w_t.r_biodiv in
    let r_t1  = w_t1.r_biodiv in
    let gold  = corridors.rbiodiversity.goldmax in
    if r_t < gold && r_t1 >= gold then
      require w_t1.DeployDecision != "PROD"
      require w_t1.E_adj <= w_t.E_adj
      require w_t1.R_raw >= w_t.R_raw
    end
```

These invariants mirror the existing safestep and override monotonicity rules, but scoped to specific ecological planes.

### 3.5 Invariant: dual residual safestep (V, U)

The dual residual extends the Lyapunov argument to uncertainty and enforces non‑expansiveness in both physical risk and data‑trust.

```aln
table uncertaintyvector
  col r_calib  double  maps varid rcalib
  col r_sigma  double  maps varid rsigma


invariant SafestepDual
  kind lyapunov
  applies_to (riskvector, uncertaintyvector)
  for_all transitions (rv_t, uv_t) -> (rv_t1, uv_t1) over same nodeid
    let V_t   = LyapunovResidual(rv_t)
    let V_t1  = LyapunovResidual(rv_t1)
    let U_t   = UncertaintyResidual(uv_t)
    let U_t1  = UncertaintyResidual(uv_t1) in
    if V_t > corridors.V_interior_max then
      require V_t1 <= V_t
    end
    if U_t > corridors.U_interior_max then
      require U_t1 <= U_t
    end
```

`LyapunovResidual` and `UncertaintyResidual` are bound to the C ecosafety core implementation at link time; the ALN spec treats them as pure functions over the vectors.

## 4. Replay CI hooks and discoverability

To make these structures not just defined but enforced, the associated CI configuration should:

- Register replay profiles under a discoverable prefix, for example
  - `replay/PhoenixMAR2026v1/`
  - `replay/PhoenixCanal2026v1/`
- Bind each replay profile to the `DataTrustAndPlanesPhoenix2026v1` particle and the `particlesCyboquaticDataTrustPhoenix2026v1.csv` shard.
- Reference the invariants `BlockedByCalib`, `BlockedBySigma`, `NoOffsetAboveGold_*`, and `SafestepDual` as hard CI gates.

A convention that keeps everything easy to find is:

- All Phoenix‑wide ecosafety particles and shards live under `qpudatashards/`.
- Each particle targeting Phoenix and ecosafety has a `Phoenix2026v1` suffix in both filename and `meta.version`.
- Replay profiles for Phoenix reference these particles by basename, so that tooling can discover the correct ALN and CSV pairs by pattern matching.

## 5. Summary of structuring improvements

The qpudatashard structuring improvements encoded in this document are:

1. A single, high‑profile CSV shard `particlesCyboquaticDataTrustPhoenix2026v1.csv` carrying rcalib, rsigma, the extended ecological planes, and KER values per Phoenix node and time window.
2. A companion ALN manifest `DataTrustAndPlanesPhoenix2026v1.aln` that defines corridors, risk vectors, KER windows, non‑compensability rules, and dual residual safestep, all wired to the data‑trust and ecological planes.
3. A consistent placement at `qpudatashards/` with Phoenix‑specific naming, making the particles maximally discoverable by both humans and automated tooling.
