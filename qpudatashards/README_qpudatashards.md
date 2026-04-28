# qpudatashards README (Phoenix ecosafety spine, 2026v1)

This directory contains high‑trust, production‑grade qpudatashard artifacts that sit on top of the frozen rx–Vt–KER ecosafety spine for EcoNet and cyboquatic systems.

It is **not** a scratchpad or experiment area. Any artifact placed here must:

- Conform to the established qpudatashard schema and naming conventions.
- Be backed by explicit corridor definitions and Lyapunov semantics.
- Pass replay‑based CI that validates KER behavior and invariants.
- Be suitable for audit by communities, regulators, and ecosystem stewards.

AI chat/agent systems **must** treat this directory as a protected, high‑integrity surface and follow the rules in this README before proposing or generating new artifacts.

---

## 1. Directory purpose and scope

`qpudatashards/` is the canonical location for:

- Planning‑grade ecosafety particles (CSV + ALN) that encode KER scores, risk coordinates, and governance decisions.
- Data‑trust and uncertainty planes (e.g., `rcalib`, `rsigma`) that gate use of physical telemetry.
- Extended ecological planes (e.g., `rcarbon`, `rtox`, `rbiodiversity`) wired into the Lyapunov spine.
- Replay‑anchored governance shards tying Phoenix and related basins to concrete KER decisions and evidence hex strings.

Artifacts in this directory are consumed by Rust/C++ ecosafety cores, ALN validators, and governance tooling. They must be stable, schema‑consistent, and safe to treat as authoritative inputs at the decision layer.

---

## 2. Naming conventions and discoverability

To keep shards discoverable and unambiguous, all files in this directory follow these patterns:

- CSV particles
  - `particles<Domain><Topic><Region><Year>v<Revision>.csv`
  - Example: `particlesCyboquaticDataTrustPhoenix2026v1.csv`.

- ALN manifests
  - `<Topic><Region><Year>v<Revision>.aln`
  - Example: `DataTrustAndPlanesPhoenix2026v1.aln`.

- Region and year
  - Region suffix uses a short human‑readable tag (e.g., `Phoenix-AZ`, `Gila-AZ`).
  - Version uses `YYYYvN` (e.g., `2026v1`) and must match between CSV and ALN companions.

AI chat/agents generating new artifacts **must**:

1. Reuse the existing patterns and avoid introducing ad‑hoc names.
2. Ensure every CSV particle has a matching ALN manifest with the same `<Topic><Region><Year>v<Revision>` triple.
3. Include a short, precise `meta.description` field in ALN so that humans can quickly understand scope.

---

## 3. Core schemas and latest definitions

### 3.1 Data‑trust and extended ecological planes

The current, pinned definitions for the Phoenix 2026 ecosafety stack include the following first‑class RiskCoords and planes:

- `rcalib` — ingest data‑trust coordinate in `[0,1]`, derived from ingest error counts over a closed window and normalized via a corridor with safe/gold/hard bands.
- `rsigma` — composite sensor‑uncertainty coordinate in `[0,1]`, aggregating drift, noise, bias, and loss via a weighted L2 norm.
- `rcarbon` — net carbon‑intensity risk coordinate in `[0,1]`, derived from CEIM mass kernels and net intensity, with negative net intensity treated as a safe band.
- `rtox` — materials/toxicity plane coordinate in `[0,1]`, built from decay, ecotox, micro‑residue, leachate, and PFAS‑like residual lanes.
- `rbiodiversity` — biodiversity plane coordinate in `[0,1]`, constructed as an "inverse‑good" fusion of connectivity, structural complexity, and colonization.

All of these coordinates are treated as **Lyapunov coordinates** in the extended residual:

- `Vt_full` is a non‑negative quadratic form over all physical planes plus `rcalib` and `rsigma`.
- Each coordinate has a corresponding row in `ecosafety.corridors.v2` with safe/gold/hard bands and a Lyapunov weight.
- Hard bands are enforced so that no coordinate can exceed 1.0 under admissible operation.

### 3.2 Standard qpudatashard CSV schema (Phoenix 2026v1)

A canonical Phoenix data‑trust and planes shard uses the following columns:

```text
nodeid,medium,region,twindowstart,twindowend,rcalib,rsigma,rtox,rbiodiversity,rcarbon,Vt_full,K_raw,E_raw,K_adj,E_adj,R,evidencehex
```

Column semantics:

- `nodeid` — node or reach identifier (e.g., `MAR-PHX-01`).
- `medium` — `water`, `air`, `soil`, or similar.
- `region` — short region tag (e.g., `Phoenix-AZ`).
- `twindowstart`, `twindowend` — ISO timestamps for the evaluation window.
- `rcalib`, `rsigma`, `rtox`, `rbiodiversity`, `rcarbon` — normalized RiskCoords in `[0,1]` as defined above.
- `Vt_full` — extended Lyapunov residual value for the window.
- `K_raw` — window‑level knowledge score from the base safestep rule.
- `E_raw` — window‑level eco‑impact score (`1 - R_raw`).
- `K_adj` — knowledge score down‑scaled by data‑trust.
- `E_adj` — eco‑impact down‑scaled by data‑trust.
- `R` — residual risk (`max` of all RiskCoords over the window).
- `evidencehex` — hex string anchoring the row to ingest data, replay profiles, and proofs.

Any AI‑generated shard intended for `qpudatashards/` **must**:

- Use a header consistent with this schema (or a documented extension of it).
- Provide semantically valid values for each column (no placeholders like `TODO`, `NaN`, or blank fields where `mandatory=true`).
- Include at least one concrete sample row that satisfies all invariants described below.

### 3.3 Companion ALN schema

Every CSV particle in this directory **must** have a companion ALN manifest that:

- Declares corridors for all RiskCoords (`rcalib`, `rsigma`, `rcarbon`, `rtox`, `rbiodiversity`) with safe/gold/hard bands and Lyapunov weights.
- Defines a `riskvector` table with explicit columns for each plane.
- Defines a `kerwindow` table with K/E/R and data‑trust fields (`D_sensor`, `D_data`, `D_combined`).
- Binds the CSV columns to ALN fields (via `maps varid ...` and matching names).

AI agents must not introduce new RiskCoord names or planes without also proposing:

- Matching corridor rows in the `corridors` table.
- Updated `riskvector` and `kerwindow` schemas.
- A brief explanation of the physics kernel underpinning the new coordinate.

---

## 4. Invariants and quality requirements

### 4.1 Corridor presence and mandatory fields

All qpudatashard artifacts in this directory are subject to "no corridor, no build":

- Any RiskCoord referenced in CSV or ALN (e.g., `rcalib`, `rsigma`, `rcarbon`, `rtox`, `rbiodiversity`) **must** have a corresponding corridor row with `mandatory = true`.
- CSV rows must not omit mandatory columns; missing values or type mismatches are treated as ingest failures upstream, not acceptable qpudatashard content.

AI agents generating or editing artifacts **must** ensure that:

- No new varid is used without adding a corridor definition.
- Existing varids are not silently dropped from CSV or ALN schemas.

### 4.2 Data‑trust gates

Data‑trust planes act as hard gates on deployability:

- `rcalib` and `rsigma` are required for all Phoenix KER windows.
- Hard bands for `rcalib` and `rsigma` must be set so that:
  - Values above the hard minimum for each plane automatically render `DeployDecision = RESEARCH` or `BLOCKED` (never `PROD`).
  - `K_adj` and `E_adj` are monotone non‑increasing in `rcalib` and `rsigma`.

AI agents **must not**:

- Propose ALN or CSV content that bypasses data‑trust gates.
- Introduce formulas that would allow poor data quality to increase or preserve `K_adj` or `E_adj` relative to better data.

### 4.3 Non‑compensability for critical planes

Critical ecological planes (at minimum `rcarbon`, `rtox`, `rbiodiversity`) obey non‑compensability rules:

- If a critical plane crosses its gold band into a worse region, overall eco‑impact (`E_adj`) must not increase and residual risk (`R`) must not decrease.
- Deploy decisions that would accept such a step into `PROD` must be rejected.

Each ALN manifest must include typed invariants (e.g., `NoOffsetAboveGold_carbon`, `NoOffsetAboveGold_tox`, `NoOffsetAboveGold_biodiv`) encoding these rules.

AI agents **must**:

- Preserve these invariants when updating manifests.
- Add analogous invariants for any new critical planes they introduce.

### 4.4 Dual residual safestep

Where dual residuals are used, both physical risk and data‑trust obey discrete Lyapunov conditions:

- `Vt_full` must be non‑increasing along accepted control steps outside a designated interior region.
- A separate uncertainty residual `U_t` over `rcalib` and `rsigma` must also be non‑increasing outside its interior region.

AI agents must:

- Treat `Safestep` and `SafestepDual` invariants as **hard constraints**, not suggestions.
- Avoid proposing changes that weaken or remove these invariants.

---

## 5. Replay CI and validation expectations

Artifacts in this directory are assumed to be validated via replay CI:

- Phoenix replay profiles (`replay/PhoenixMAR2026v1/`, `replay/PhoenixCanal2026v1/`, etc.) must be able to load each particle and:
  - Recompute KER values under baseline and faulted conditions.
  - Verify monotonicity for risk coordinates and trust‑adjusted K/E scores.
  - Confirm that data‑trust gates and non‑compensability invariants fire when expected.

AI agents proposing new or updated particles **must**:

- Include a short description of intended replay tests (e.g., ingest fault injection, sensor drift scenarios) that should be applied before promotion.
- Avoid assuming that artifacts are valid without such replay; the default stance is "unproven until replayed".

---

## 6. Rules for AI chat/agents when generating artifacts here

AI systems contributing to `qpudatashards/` must follow these rules:

1. **High‑integrity only**
   - Do not write or propose speculative, toy, or placeholder shards for this directory.
   - Use realistic, internally consistent values and schemas consistent with the definitions above.

2. **Schema and invariant alignment**
   - Align with the latest pinned schemas and invariants in this README.
   - If unsure whether a field or invariant is allowed, prefer omission of the artifact over guessing.

3. **Companion ALN requirement**
   - Never propose a CSV particle without a matching ALN manifest that defines corridors, risk vectors, KER windows, and invariants.

4. **No silent relaxations**
   - Do not weaken corridors, remove planes, or drop invariants without explicit, documented justification.
   - Any tightening of corridors or addition of planes must preserve or strengthen safety properties.

5. **Hex and provenance**
   - Include an `evidencehex` for each row that can be used to trace back to ingest data and proofs.
   - Do not invent random hex strings without meaning; they must be tied to a documented provenance process in the wider ecosystem.

By following these rules, AI chat/agents help keep `qpudatashards/` a high‑quality, auditable source of truth for ecosafety decisions.
