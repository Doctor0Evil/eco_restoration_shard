// Filename: crates/eco-ci-validate/src/main.rs
// Destination: crates/eco-ci-validate/src/main.rs

use std::env;
use std::path::PathBuf;
use std::process;

use ecosafety_core::{
    invariants::{corridor_present, safestep},
    ker::KerScores,
    residual::Residual,
};
use eco_shards::{
    loaders::{load_shard_file, ShardFormat},
    types::{CorridorBands, EcoShard},
};

fn main() {
    // Simple CLI:
    //   eco-ci-validate shards/ --min-k 0.90 --min-e 0.90 --max-r 0.13
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: eco-ci-validate <shard-path> [--min-k K] [--min-e E] [--max-r R]");
        process::exit(1);
    }

    let shard_root = PathBuf::from(&args[1]);
    let mut min_k = 0.90;
    let mut min_e = 0.90;
    let mut max_r = 0.13;

    let mut i = 2;
    while i + 1 < args.len() {
        match args[i].as_str() {
            "--min-k" => {
                min_k = args[i + 1].parse().unwrap_or(min_k);
            }
            "--min-e" => {
                min_e = args[i + 1].parse().unwrap_or(min_e);
            }
            "--max-r" => {
                max_r = args[i + 1].parse().unwrap_or(max_r);
            }
            _ => {}
        }
        i += 2;
    }

    let mut failed = false;

    // Walk shard directory (csv, json, aln, yaml) and validate each ecosafety shard.
    for entry in walkdir::WalkDir::new(&shard_root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();

        let format = match ShardFormat::from_path(path) {
            Some(f) => f,
            None => continue,
        };

        match load_shard_file(path, format) {
            Ok(shard) => {
                if let Err(err) = validate_shard(path, &shard, min_k, min_e, max_r) {
                    eprintln!("[eco-ci-validate] FAIL: {} -> {}", path.display(), err);
                    failed = true;
                }
            }
            Err(e) => {
                eprintln!(
                    "[eco-ci-validate] FAIL: {} -> unable to parse shard: {}",
                    path.display(),
                    e
                );
                failed = true;
            }
        }
    }

    if failed {
        process::exit(2);
    }
}

fn validate_shard(
    path: &std::path::Path,
    shard: &EcoShard,
    min_k: f64,
    min_e: f64,
    max_r: f64,
) -> Result<(), String> {
    // 1. mandatory corridors present (no corridor, no build).
    if !corridor_present(&shard.corridors) {
        return Err("corridor_present invariant failed (missing mandatory corridors)".into());
    }

    // 2. non-increasing residual risk and no hard-band breach (violated corridor deratestop).
    let prev_res = shard.risk_state.prev_residual.clone().unwrap_or_else(|| Residual {
        vt: shard.risk_state.vt,
        coords: shard.risk_state.coords.clone(),
    });
    let next_res = Residual {
        vt: shard.risk_state.vt,
        coords: shard.risk_state.coords.clone(),
    };

    let decision = safestep(&prev_res, &next_res);
    if decision.stop || decision.derate {
        return Err(format!(
            "safestep invariant failed: derate={}, stop={}, reason={}",
            decision.derate, decision.stop, decision.reason
        ));
    }

    // 3. KER gates for production-eligible shards.
    if shard.governance.production_eligible {
        let ker = KerScores::from_shard(shard)?;
        if ker.knowledge_factor < min_k {
            return Err(format!(
                "K too low: {:.3} < {:.3}",
                ker.knowledge_factor, min_k
            ));
        }
        if ker.eco_impact < min_e {
            return Err(format!("E too low: {:.3} < {:.3}", ker.eco_impact, min_e));
        }
        if ker.risk_of_harm > max_r {
            return Err(format!(
                "R too high: {:.3} > {:.3}",
                ker.risk_of_harm, max_r
            ));
        }
    }

    Ok(())
}
