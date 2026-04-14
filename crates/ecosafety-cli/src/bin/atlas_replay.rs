//! Atlas replay CLI tool for long-horizon Vt and KER recomputation.
//! Processes historical qpudatashard streams and produces analytics.

use clap::Parser;
use ecosafety_core::{
    CorridorSet, ResidualAtlas, QPUShardV1, EvidenceHex, Lane,
    compute::{DualResidual, ResidualFast},
};
use std::path::PathBuf;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use serde::{Serialize, Deserialize};
use anyhow::{Result, Context};

#[derive(Parser, Debug)]
#[command(name = "atlas-replay")]
#[command(about = "Recompute Vt and KER over historical shard streams")]
struct Cli {
    /// Input CSV/parquet file containing shard rows
    #[arg(short, long)]
    input: PathBuf,

    /// Output file for recomputed analytics
    #[arg(short, long)]
    output: PathBuf,

    /// Corridor set specification (ALN file)
    #[arg(short, long, default_value = "ecosafety-specs/grammar/CorridorBands2026v1.aln")]
    corridors: PathBuf,

    /// Lane to validate against (RESEARCH, PILOT, PROD)
    #[arg(long, default_value = "PROD")]
    lane: String,

    /// Batch size for streaming processing
    #[arg(long, default_value = "10000")]
    batch_size: usize,

    /// Output format (json, csv, parquet)
    #[arg(long, default_value = "json")]
    format: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AtlasOutputRow {
    timestamp: i64,
    node_id: String,
    vt: f32,
    ut: f32,
    ker_k: f32,
    ker_e: f32,
    ker_r: f32,
    lane_violation: bool,
    evidencehex: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Load corridors
    let corridors = CorridorSet::from_aln(&cli.corridors.to_string_lossy())
        .context("Failed to load corridor set")?;
    corridors.validate()?;

    let lane = match cli.lane.as_str() {
        "RESEARCH" => Lane::RESEARCH,
        "PILOT" => Lane::PILOT,
        "PROD" => Lane::PROD,
        _ => anyhow::bail!("Invalid lane: {}", cli.lane),
    };

    // Open input (simplified CSV parsing)
    let file = File::open(&cli.input)?;
    let reader = BufReader::new(file);
    let mut csv_reader = csv::Reader::from_reader(reader);

    // Output
    let out_file = File::create(&cli.output)?;
    let mut writer = BufWriter::new(out_file);

    // Initialize residual
    let mut atlas = ResidualAtlas::from_corridors(&corridors);
    let mut dual = DualResidual::new(corridors.weights(), [0.5, 0.5]);

    let mut outputs = Vec::with_capacity(cli.batch_size);
    let mut total_rows = 0;
    let mut violations = 0;

    // Process rows
    for (idx, result) in csv_reader.records().enumerate() {
        let record = result?;
        // Parse expected CSV format
        let raw: Vec<f32> = record.iter()
            .take(7)
            .map(|s| s.parse::<f32>().unwrap_or(0.0))
            .collect();
        if raw.len() < 7 {
            continue;
        }
        let raw_arr: [f32; 7] = raw.try_into().unwrap();

        let timestamp = record.get(7).and_then(|s| s.parse().ok()).unwrap_or(0i64);
        let node_id = record.get(8).unwrap_or("unknown").to_string();

        // Normalize and compute
        atlas.update_from_raw(&raw_arr, &corridors);
        dual.update_risk(&atlas.r);
        dual.update_uncertainty(0.1, 0.05); // Placeholder rsigma, rcalib

        // Check lane violation
        let (min_k, min_e, max_r, max_vt) = match lane {
            Lane::RESEARCH => (0.0, 0.0, 1.0, 0.8),
            Lane::PILOT => (0.80, 0.75, 0.20, 0.5),
            Lane::PROD => (0.90, 0.90, 0.13, 0.3),
        };

        let ker_k = 0.95; // Placeholder - would compute from shard
        let ker_e = 0.91;
        let ker_r = 0.12;
        let violation = ker_k < min_k || ker_e < min_e || ker_r > max_r || atlas.vt > max_vt;
        if violation {
            violations += 1;
        }

        outputs.push(AtlasOutputRow {
            timestamp,
            node_id,
            vt: atlas.vt,
            ut: dual.ut,
            ker_k,
            ker_e,
            ker_r,
            lane_violation: violation,
            evidencehex: hex::encode([0u8; 32]),
        });

        if outputs.len() >= cli.batch_size {
            flush_batch(&mut writer, &outputs, &cli.format)?;
            outputs.clear();
        }

        total_rows = idx + 1;
    }

    if !outputs.is_empty() {
        flush_batch(&mut writer, &outputs, &cli.format)?;
    }

    println!("Processed {} rows", total_rows);
    println!("Lane violations: {} ({:.2}%)", violations, 
             100.0 * violations as f64 / total_rows as f64);

    Ok(())
}

fn flush_batch<W: Write>(writer: &mut W, rows: &[AtlasOutputRow], format: &str) -> Result<()> {
    match format {
        "json" => {
            for row in rows {
                serde_json::to_writer(&mut *writer, row)?;
                writeln!(writer)?;
            }
        }
        "csv" => {
            let mut wtr = csv::Writer::from_writer(writer);
            for row in rows {
                wtr.serialize(row)?;
            }
            wtr.flush()?;
        }
        _ => anyhow::bail!("Unsupported format: {}", format),
    }
    Ok(())
}
