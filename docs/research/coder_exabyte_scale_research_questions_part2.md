# 100 Additional Research Questions, Definition Requests, Detail Queries, and Objection Identifiers for eco_restoration_shard Coder Tooling (Part 2)

**Filename:** `coder_exabyte_scale_research_questions_part2.md`  
**Destination:** `docs/research/coder_exabyte_scale_research_questions_part2.md`

---

This document provides an additional 100 focused items, continuing from the initial 100, to further advance the **coder experience** for `eco_restoration_shard` at exabyte scale. The emphasis remains on `/mnt/oss` VFS operations, handling hundreds of files simultaneously, and maintaining ecosafety grammar, KER scoring, and Rust invariants. These items target gaps identified after the first round of answers—areas where more definition, research, or objection identification is needed to complete the project's operational readiness.

Labeling:
- **RQ** – Research Question
- **DR** – Definition Request
- **DQ** – Detail Query
- **OI** – Objection Identifier

---

1. **DQ** – What is the exact I/O concurrency limit of `ossfs` under sustained 1000+ simultaneous `open_shard()` calls from `eco-ci-validate` before read latency exceeds 2 seconds?
2. **RQ** – How should the shard streaming iterator be augmented to support cancellation and timeout semantics, so that a stuck shard read does not block the entire coder's IDE session?
3. **OI** – The placeholder generator script writes 120 crates; if a coder accidentally runs it twice, it may overwrite or duplicate entries. What safeguards prevent workspace corruption?
4. **DR** – Define "exabyte-scale storage node" in the context of `storage_shards` and `ComputeNodeShard`—what minimal hardware specification qualifies as an exabyte-capable node?
5. **RQ** – What is the optimal fan-out factor for shard directory sharding by DID prefix to maintain sub-millisecond `stat()` latency across 1 billion files?

6. **DQ** – How does `oss_vfs` handle extended attributes (xattrs) for shard files, and are they used to cache KER summary data for faster access?
7. **OI** – The `eco-ci-validate` binary recomputes KER for every shard; at exabyte scale, this will consume significant CPU. Should a precomputed KER cache shard be used, and how is it invalidated?
8. **RQ** – Can `cargo` be extended with a "shard dependency" resolver that rebuilds a crate only if its upstream shard's KER score has changed beyond a threshold?
9. **DR** – Provide a formal definition of "Lyapunov channel" numbers (e.g., "10", "11") in `CorridorBands`—how are they assigned, and what does the numeric value encode?
10. **DQ** – In `ecosafety_core::invariants`, how is `eps_safe` (the soft interior threshold for residual) determined? Is it a fixed constant or dynamically derived from corridor weights?

11. **RQ** – How can we implement a `shard-archiver` tool that safely moves old `RESEARCH` lane shards to cold storage while preserving their `evidencehex` and KER for audit purposes?
12. **OI** – The `agent_interface` restricts AI actions to three verbs; does this limit the AI's ability to propose multi-file refactors, and if so, how can we extend the vocabulary without compromising safety?
13. **DR** – Define the exact structure of the "KER summary" JSON file produced by `kersummary`—what fields beyond K, E, R, and status are required for integration with `build_scheduler`?
14. **DQ** – In `eco_shards::response_turn`, is the `promotion_reason` field free-form text or a controlled vocabulary? What values are allowed?
15. **RQ** – What profiling tools are recommended to measure memory fragmentation in long-running `eco-ci-validate` processes that continuously scan millions of shards?

16. **OI** – The `setup_mnt_oss.sh` script assumes a Unix environment; how should Windows coders (using WSL2 or native) replicate the `/mnt/oss` VFS layout without bind mounts?
17. **DR** – Provide a precise definition of "node family" in `EcoCoreParameters`—what are the valid values, and how do they affect corridor selection?
18. **DQ** – How are `CorridorBands` weights determined? Is there a formal process for assigning initial weights, or are they purely empirical?
19. **RQ** – Can we implement a "shard signature verification" service that checks Bostrom DID signatures on shard files at read time, and how would this impact coder latency?
20. **OI** – The placeholder files are stubs; what is the expected timeline for a coder to replace them with real implementations, and how does CI track placeholder aging?

21. **DR** – Define the term "KER context" more precisely—what exact information is included in `KERCONTEXT=PhoenixEcoSafetySpine2026v1`, and how does it differ from `ALNSPECHASH`?
22. **DQ** – In `eco_shards::biodegradable_substrate`, how are `LcmsAnalyte` concentration values normalized into `rtox` when multiple analytes are present? Is there a summation or maximum rule?
23. **RQ** – How can we implement a "shard replay" mode for `eco-ci-validate` that validates historical shards against current corridor tables to detect silent corridor drift?
24. **OI** – The `eco-ci-build.sh` script uses `set -euo pipefail`; if a coder's environment lacks `bash`, how should they invoke the build on Windows PowerShell?
25. **DR** – Provide a definition for "risk coordinate saturation"—when a coordinate reaches its hard band, what additional governance steps are triggered beyond `Derate/Stop`?

26. **DQ** – What is the expected behavior of `oss_vfs::open_shard()` when a shard file is being concurrently written by another process? Does it return a partial read, an error, or block?
27. **RQ** – How should we design a "shard health dashboard" that displays real-time KER trends and corridor violations to coders without overwhelming them with metrics?
28. **OI** – The current design relies heavily on CSV; what are the performance implications of parsing 10,000‑column CSV files, and would a binary format (e.g., Parquet) be considered for hot paths?
29. **DR** – Define "ecosafety grammar versioning"—how are changes to ALN schemas and corridor tables versioned, and how do shards reference specific versions?
30. **DQ** – In `ecosafety_core::ker`, how is `eco_impact` (E) calculated when multiple benefit streams (e.g., carbon avoided, water saved) apply simultaneously?

31. **RQ** – Can we implement a "shard‑based feature flag" system where crate features are gated by KER scores, so unsafe or low‑K code cannot be compiled in PROD lanes?
32. **OI** – The `agent_interface` writes to `/mnt/oss/staging/`; what prevents a malicious or buggy agent from flooding staging with millions of invalid shards, causing disk exhaustion?
33. **DR** – Provide a definition for "shard family lifecycle"—from creation (RESEARCH) to archival (ARCHIVE), what are the required steps and approvals?
34. **DQ** – How does `eco_shards` handle CSV files with BOM (byte order mark) or non‑UTF8 encodings? Are such files rejected or normalized?
35. **RQ** – What is the best practice for testing the `oss_vfs` crate under network partition scenarios where `/mnt/oss` becomes temporarily unavailable?

36. **OI** – The placeholder crates are all named `mod_XXX`; what naming convention should coders use when renaming them to meaningful domain names to avoid collisions?
37. **DR** – Define "corridor calibration evidence"—what specific types of data (lab reports, pilot telemetry, peer‑reviewed studies) are acceptable to justify tightening a corridor band?
38. **DQ** – In `eco-ci-validate`, are corridor tables loaded once at startup or per shard? What is the memory footprint of holding all corridor bands for all regions?
39. **RQ** – How can we implement a "shard‑based lint" for Rust code that checks for patterns known to increase R (e.g., unbounded loops, large allocations) and emits warnings with KER impact estimates?
40. **OI** – The `build_scheduler` mentions Lyapunov‑gated approval; what happens if a build is approved but a later shard reveals a corridor violation after deployment? Is there a rollback mechanism?

41. **DR** – Provide a precise definition of "Shard Completeness" values—what are the allowed values (`SIMULATED`, `MEASURED`, `VERIFIED`, etc.), and how do they affect KER and lane eligibility?
42. **DQ** – In `ecosafety_core::corridor`, the `normalize_coord` function uses a piecewise linear map. What is the exact formula when the metric is "higher‑is‑better" (e.g., degradation speed)?
43. **RQ** – Can we design a `shard-fsck` tool that scans `/mnt/oss` for orphaned shards, missing index entries, or hash mismatches, similar to filesystem consistency checks?
44. **OI** – The `.envrc` file sets `RUST_ROOT`; this variable is not used by standard Rust tooling. Should it be renamed to avoid confusion, or removed entirely in favor of `CARGO_HOME`?
45. **DR** – Define "node capacity" in `StorageAndComputeNodes2026v1.aln`—what fields specify storage and compute limits, and how are they used in scheduling?

46. **DQ** – How are `rx_map_json` field updates synchronized across multiple concurrent writers to the same shard family? Is there a locking or merge strategy?
47. **RQ** – What is the optimal strategy for garbage collecting old `RESEARCH` and `SIM` lane shards that are no longer needed, while preserving auditability?
48. **OI** – The `eco-ci-validate` binary currently scans all shards; for a coder working on a single crate, this is overkill. How can we scope validation to only the shards relevant to the current change?
49. **DR** – Provide a definition for "ecosafety invariant orthogonality"—how do we ensure that tightening one corridor does not inadvertently relax another?
50. **DQ** – In `oss_vfs`, what is the behavior of `list_shards()` when the underlying object store returns an incomplete listing due to eventual consistency?

51. **RQ** – How can we integrate `cargo‑audit` and `cargo‑deny` into the ecosafety CI pipeline to detect vulnerable or unlicensed dependencies, and should this affect KER scores?
52. **OI** – The placeholder script creates VFS snapshot stubs; are these stubs ever validated, or are they purely documentation? If the latter, they may become outdated and misleading.
53. **DR** – Define the exact format of the "improvement principle trails" stored in the 10 TB audit reserve—what fields are recorded per improvement, and how are they indexed?
54. **DQ** – In `eco_shards::loaders`, how are CSV header rows validated against ALN schemas? What happens if a column is missing or out of order?
55. **RQ** – What is the best approach to implement a "shard replication" mechanism across multiple geographic regions to ensure high availability of `/mnt/oss` for distributed teams?

56. **OI** – The `agent_interface` vocabulary is limited to three actions; this may force agents to simulate complex workflows through many small steps, increasing shard volume. Is this acceptable at exabyte scale?
57. **DR** – Provide a definition for "KER target met"—is it a boolean derived from thresholds, or can it have partial states (e.g., "K met, E not met")?
58. **DQ** – How does `ecosafety_core::residual` handle the case where a coordinate's corridor band is missing entirely? Does it default to a conservative value or fail loudly?
59. **RQ** – Can we implement a "shard‑aware debugger" that allows coders to step through shard validation logic and inspect intermediate rx, Vt, and KER values?
60. **OI** – The `eco-ci-build.sh` wrapper hardcodes the crate name `eco-ci-validate`; what if a coder wants to validate a different binary or workspace member?

61. **DR** – Define "ecosafety corridor update" process—what steps must a coder follow to propose, test, and merge a change to `EcoCoreParameters2026v1.csv`?
62. **DQ** – In `response_shard_core`, are `knowledge_factor`, `eco_impact`, and `risk_of_harm` stored with a fixed number of decimal places? How does rounding affect `ker_deployable` decisions?
63. **RQ** – How should we design a "shard migration rollback" procedure in case a migration tool introduces errors that increase R across many shards?
64. **OI** – The `storage_shards` crate includes `is_safe_for_writes()`; what is the expected behavior when a node is at the edge of its hard band but a critical security patch must be deployed?
65. **DR** – Provide a definition for "Bostrom DID method" as used in shard signatures—what is the exact DID prefix and resolution method?

66. **DQ** – In `eco_shards::response_turn`, is `Vt_before` required to exactly match the `Vt_after` of the previous shard in the sequence? What tolerance is allowed?
67. **RQ** – Can we implement a "shard‑based code coverage" tool that correlates test coverage with KER improvements, showing which code paths most affect ecosafety?
68. **OI** – The placeholder script generates ALN files with simple key‑value format; if a coder forgets to upgrade them to full HALN, will CI catch this, or could invalid specs slip through?
69. **DR** – Define "evidence hex stamp" algorithm more concretely—if it includes spec content and protocol ID, what is the exact serialization and hash function (e.g., BLAKE3)?
70. **DQ** – How does `oss_vfs` handle file locking? Is there any advisory or mandatory locking to prevent concurrent writes to the same shard file?

71. **RQ** – What is the optimal shard file size distribution for exabyte‑scale storage to balance listing latency and throughput? Should very small shards be coalesced?
72. **OI** – The `eco-ci-validate` binary uses `serde_yaml`; YAML is known for security issues with deserialization. Is this dependency safe in a CI context, or should it be replaced with a simpler config format?
73. **DR** – Provide a definition for "corridor mandatory flag"—what criteria make a corridor mandatory versus optional, and how is this decided?
74. **DQ** – In `ecosafety_core::invariants::safestep`, is the decision `Derate` or `Stop` based solely on `Vt` increase, or does it also consider the magnitude of increase?
75. **RQ** – How can we implement a "shard heatmap" visualization for coders that shows which topics or regions have the highest R and lowest K/E, guiding research priorities?

76. **OI** – The `agent_interface` restricts writes to staging; how does the promotion process handle shards that reference other shards (e.g., via `corridorid`)? Must all referenced shards also be promoted together?
77. **DR** – Define "Lyapunov weight" selection process—how are initial weights chosen for new risk coordinates, and how are they recalibrated over time?
78. **DQ** – In `eco_shards::biodegradable_substrate`, how are `mass_loss_28d` and `mass_loss_180d` normalized into `rbiodegspeed`? What is the mapping formula?
79. **RQ** – Can we design a "shard‑based time travel" query interface that allows coders to see the KER landscape at any past commit, for debugging regressions?
80. **OI** – The `setup_mnt_oss.sh` script uses `sudo`; in containerized CI environments where `sudo` is unavailable, how should the VFS be set up?

81. **DR** – Provide a definition for "ecosafety lane promotion"—what is the exact procedure and required approvals to move a shard family from RESEARCH to PILOT, and from PILOT to PROD?
82. **DQ** – In `eco_shards::response_turn`, how are `corridor_update_ids` and `equation_update_ids` encoded? Are they comma‑separated lists or JSON arrays?
83. **RQ** – How can we implement a "shard deduplication" service that identifies identical shards (same content hash) and replaces them with hard links or references to save storage?
84. **OI** – The placeholder crates include `#![forbid(unsafe_code)]`, but the coders may need `unsafe` for FFI to interact with hardware. How should such crates be marked and governed?
85. **DR** – Define "ecosafety audit trail" retention policy—for how long must decision logs and corridor change shards be kept, and in what format?

86. **DQ** – In `ecosafety_core::residual`, is `Vt` computed as a simple sum or a more complex aggregation (e.g., moving average) for windowed KER?
87. **RQ** – What is the best practice for monitoring `/mnt/oss` disk usage and alerting when free space falls below a threshold, given the 16E capacity?
88. **OI** – The `eco-ci-validate` binary exits with code 2 on failure; how does this integrate with `git bisect` or other developer tools that expect standard exit codes?
89. **DR** – Provide a definition for "shard index" as referenced in `/mnt/oss/index/`—what exact information is stored per shard, and how is the index updated?
90. **DQ** – In `oss_vfs::open_shard()`, is the file path validated against the expected shard family and lane based on the path structure, or is any file under `/mnt/oss` readable?

91. **RQ** – How can we implement a "shard‑based canary deployment" where new corridor bands are first applied to a small percentage of traffic and only rolled out if R does not increase?
92. **OI** – The `agent_interface` vocabulary includes `ProposePatch`; does this cover changes to ALN spec files, or is there a separate action for spec changes?
93. **DR** – Define "ecosafety kernel version"—how are changes to the normalization and residual kernels (e.g., from sum of squares to max) versioned and rolled out?
94. **DQ** – In `eco_shards::eco_core_parameters`, how are corridor rows with duplicate `varid` for the same region handled? Is it an error, or does the latest version take precedence?
95. **RQ** – Can we implement a "shard‑based IDE plugin" that displays KER badges next to code files, showing the ecosafety impact of the code based on associated shards?

96. **OI** – The placeholder script generates 120 crates; if the coder wants only a subset (e.g., 10), is there a parameter to control the count, or must they manually delete the extras?
97. **DR** – Provide a definition for "risk coordinate direction"—how is the `direction` field in `CorridorBands` used in normalization, and what are the allowed values?
98. **DQ** – In `eco-ci-validate`, how are shard validation errors aggregated? Is there a limit on the number of errors reported before the tool exits?
99. **RQ** – How should we design a "shard backup and restore" procedure that ensures all `evidencehex` and DIDs remain verifiable after restoration to a new `/mnt/oss` instance?
100. **OI** – The current documentation assumes coders are familiar with ecosafety grammar and KER; what is the onboarding plan for new contributors, and where are the learning materials stored as shards?
