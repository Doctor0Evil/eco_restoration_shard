# FILE .tools\scaffold\New-CodeCommandPlaceholders.ps1
param(
    [int]$CrateCount = 120,
    [int]$ShardCount = 120,
    [string]$RepoRoot = "$PSScriptRoot\..\.."
)

$ErrorActionPreference = "Stop"

# 1. Create placeholder Rust crates under crates/mod_XXX
1..$CrateCount | ForEach-Object {
    $name = "mod_{0:000}" -f $_
    $dir  = Join-Path $RepoRoot "crates\$name"
    $src  = Join-Path $dir "src"

    New-Item -ItemType Directory -Path $src -Force | Out-Null

    $cargoToml = @"
[package]
name = "$name"
version = "0.1.0"
edition = "2021"

[dependencies]
"@

    $libRs = @"
#![forbid(unsafe_code)]

pub fn placeholder() {
    // TODO: promote this into a real cc-vfs/SITQ-aware module.
}
"@

    Set-Content -Path (Join-Path $dir "Cargo.toml") -Value $cargoToml -Encoding UTF8
    Set-Content -Path (Join-Path $src "lib.rs")     -Value $libRs   -Encoding UTF8
}

# 2. Create placeholder shard CSV/ALN pairs under shards\placeholders
$shardRoot = Join-Path $RepoRoot "shards\placeholders"
New-Item -ItemType Directory -Path $shardRoot -Force | Out-Null

1..$ShardCount | ForEach-Object {
    $id = "{0:000}" -f $_
    $csvPath = Join-Path $shardRoot "placeholder_$id.csv"
    $alnPath = Join-Path $shardRoot "placeholder_$id.aln"

    $csv = @"
ALNSHARDID,TOPIC,K,E,R,VT_BEFORE,VT_AFTER,EVIDENCEHEX
placeholder_$id,scaffolding,0.0,0.0,0.99,1.00,1.00,0xDEADBEEF$id
"@

    $aln = @"
spec_id: PlaceholderShard2026v1_$id
kind: schema
description: 'Scaffolding shard for eco_restoration_shard placeholder $id'
"@

    Set-Content -Path $csvPath -Value $csv -Encoding UTF8
    Set-Content -Path $alnPath -Value $aln -Encoding UTF8
}

# 3. Optional: create VFS snapshot stubs for cc-vfs
$vfsRoot = Join-Path $RepoRoot "vfs_snapshots\placeholders"
New-Item -ItemType Directory -Path $vfsRoot -Force | Out-Null

1..$CrateCount | ForEach-Object {
    $id = "{0:000}" -f $_
    $jsonPath = Join-Path $vfsRoot "snapshot_mod_$id.json"

    $snapshot = @"
{
  "ccvfs_id": "cc-vfs1",
  "profile": "local",
  "entries": [
    {
      "path": "crates/mod_$id/src/lib.rs",
      "content_b64": "",
      "sha": "",
      "is_dir": false
    }
  ]
}
"@

    Set-Content -Path $jsonPath -Value $snapshot -Encoding UTF8
}
