//! storage_shards — StorageNodeShard schemas for exabyte capacity planning
//!
//! This crate defines `StorageNodeShard` and `ComputeNodeShard` types that
//! describe storage racks and build machines with ecosafety risk coordinates:
//! - r_energy, r_materials, r_heat, r_sigma (storage)
//! - r_energy, r_rf, r_heat, r_sigma (compute)
//!
//! AI‑Chat can query these shards to decide where it is safe to add large
//! datasets or schedule heavy indexing/build jobs.

#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use cyboquatic_ecosafety_core::{CorridorBands, LyapunovWeights, Residual, RiskCoord, RiskVector};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Storage node shard describing one rack/node window.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StorageNodeShard {
    // Identity
    pub node_id: String,
    pub rack_id: String,
    pub region: String,
    pub lat: f64,
    pub lon: f64,
    pub t_window_start: String,
    pub t_window_end: String,

    // Raw metrics
    pub capacity_bytes: u64,
    pub used_bytes: u64,
    pub iops: u64,
    pub power_w: f64,
    pub temp_c: f64,
    pub device_count: u32,

    // Normalized risk coordinates
    pub r_energy: f64,
    pub r_materials: f64,
    pub r_heat: f64,
    pub r_sigma: f64,

    // Governance
    pub vt: f64,
    pub k_score: f64,
    pub e_score: f64,
    pub r_score: f64,
    pub lane: String,
    pub evidence_hex: String,
}

impl StorageNodeShard {
    /// Create a new shard from raw metrics, computing normalized risks and Vt.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        node_id: String,
        rack_id: String,
        region: String,
        lat: f64,
        lon: f64,
        capacity_bytes: u64,
        used_bytes: u64,
        iops: u64,
        power_w: f64,
        temp_c: f64,
        device_count: u32,
        lane: &str,
        evidence_hex: &str,
    ) -> Self {
        let t_window_start = Utc::now().to_rfc3339();
        let t_window_end = Utc::now().to_rfc3339(); // Same for point‐in‐time

        // Compute normalized risks using corridor bands
        let energy_bands = CorridorBands::new(0.0, 500.0, 2000.0); // W
        let heat_bands = CorridorBands::new(20.0, 35.0, 50.0); // °C
        let materials_bands = CorridorBands::new(0.0, 0.5, 1.0); // Fraction of lifecycle

        let r_energy = energy_bands.normalize(power_w).value();
        let r_heat = heat_bands.normalize(temp_c).value();
        let r_materials = materials_bands
            .normalize(device_count as f64 / 100.0)
            .value();
        let r_sigma = 0.05; // Base uncertainty

        // Build RiskVector and compute Vt
        let rv = RiskVector {
            energy: RiskCoord::new(r_energy),
            hydraulics: RiskCoord::new(0.0), // Not applicable
            biology: RiskCoord::new(0.0),
            carbon: RiskCoord::new(0.0),
            materials: RiskCoord::new(r_materials),
            biodiversity: RiskCoord::new(0.0),
            sigma: RiskCoord::new(r_sigma),
        };

        let weights = LyapunovWeights::default_carbon_negative();
        let vt = residual(&rv, &weights).value;

        // KER scores
        let r_score = rv.max_coord().value();
        let e_score = (1.0 - r_score).clamp(0.0, 1.0);
        let k_score = 0.95; // Assumed high for measured data

        Self {
            node_id,
            rack_id,
            region,
            lat,
            lon,
            t_window_start,
            t_window_end,
            capacity_bytes,
            used_bytes,
            iops,
            power_w,
            temp_c,
            device_count,
            r_energy,
            r_materials,
            r_heat,
            r_sigma,
            vt,
            k_score,
            e_score,
            r_score,
            lane: lane.to_string(),
            evidence_hex: evidence_hex.to_string(),
        }
    }

    /// Check if this node is in a safe band for writes.
    pub fn is_safe_for_writes(&self) -> bool {
        self.r_energy <= 0.5 && self.r_heat <= 0.5 && self.r_score <= 0.13
    }

    /// CSV header.
    pub fn csv_header() -> Vec<&'static str> {
        vec![
            "node_id",
            "rack_id",
            "region",
            "lat",
            "lon",
            "t_window_start",
            "t_window_end",
            "capacity_bytes",
            "used_bytes",
            "iops",
            "power_w",
            "temp_c",
            "device_count",
            "r_energy",
            "r_materials",
            "r_heat",
            "r_sigma",
            "vt",
            "k_score",
            "e_score",
            "r_score",
            "lane",
            "evidence_hex",
        ]
    }

    /// Convert to CSV record.
    pub fn to_csv_record(&self) -> Vec<String> {
        vec![
            self.node_id.clone(),
            self.rack_id.clone(),
            self.region.clone(),
            self.lat.to_string(),
            self.lon.to_string(),
            self.t_window_start.clone(),
            self.t_window_end.clone(),
            self.capacity_bytes.to_string(),
            self.used_bytes.to_string(),
            self.iops.to_string(),
            self.power_w.to_string(),
            self.temp_c.to_string(),
            self.device_count.to_string(),
            self.r_energy.to_string(),
            self.r_materials.to_string(),
            self.r_heat.to_string(),
            self.r_sigma.to_string(),
            self.vt.to_string(),
            self.k_score.to_string(),
            self.e_score.to_string(),
            self.r_score.to_string(),
            self.lane.clone(),
            self.evidence_hex.clone(),
        ]
    }
}

/// Compute node shard for build machines.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ComputeNodeShard {
    pub node_id: String,
    pub region: String,
    pub t_window_start: String,
    pub t_window_end: String,

    // Raw metrics
    pub cpu_util: f64,      // 0–1
    pub mem_gb: u64,
    pub power_w: f64,
    pub rf_power_density: f64, // W/m²
    pub duty_cycle: f64,    // 0–1

    // Normalized risks
    pub r_energy: f64,
    pub r_rf: f64,
    pub r_heat: f64,
    pub r_sigma: f64,

    // Governance
    pub vt: f64,
    pub k_score: f64,
    pub e_score: f64,
    pub r_score: f64,
    pub lane: String,
    pub evidence_hex: String,
}

impl ComputeNodeShard {
    /// Create a new compute shard from raw metrics.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        node_id: String,
        region: String,
        cpu_util: f64,
        mem_gb: u64,
        power_w: f64,
        rf_power_density: f64,
        duty_cycle: f64,
        lane: &str,
        evidence_hex: &str,
    ) -> Self {
        let t_window_start = Utc::now().to_rfc3339();
        let t_window_end = Utc::now().to_rfc3339();

        // Corridor bands
        let energy_bands = CorridorBands::new(0.0, 500.0, 2000.0);
        let rf_bands = CorridorBands::new(0.0, 1.0, 10.0); // W/m²
        let heat_bands = CorridorBands::new(20.0, 40.0, 60.0);

        let r_energy = energy_bands.normalize(power_w).value();
        let r_rf = rf_bands.normalize(rf_power_density).value();
        let r_heat = heat_bands
            .normalize(25.0 + cpu_util * 30.0) // Estimate temp from CPU util
            .value();
        let r_sigma = 0.05;

        let rv = RiskVector {
            energy: RiskCoord::new(r_energy),
            hydraulics: RiskCoord::new(0.0),
            biology: RiskCoord::new(0.0),
            carbon: RiskCoord::new(0.0),
            materials: RiskCoord::new(0.0),
            biodiversity: RiskCoord::new(0.0),
            sigma: RiskCoord::new(r_sigma),
        };

        let weights = LyapunovWeights::default_carbon_negative();
        let vt = residual(&rv, &weights).value;

        let r_score = rv.max_coord().value();
        let e_score = (1.0 - r_score).clamp(0.0, 1.0);
        let k_score = 0.95;

        Self {
            node_id,
            region,
            t_window_start,
            t_window_end,
            cpu_util,
            mem_gb,
            power_w,
            rf_power_density,
            duty_cycle,
            r_energy,
            r_rf,
            r_heat,
            r_sigma,
            vt,
            k_score,
            e_score,
            r_score,
            lane: lane.to_string(),
            evidence_hex: evidence_hex.to_string(),
        }
    }

    /// Check if this node is safe for heavy builds.
    pub fn is_safe_for_builds(&self) -> bool {
        self.r_energy <= 0.5
            && self.r_rf <= 0.5
            && self.r_heat <= 0.5
            && self.r_score <= 0.13
    }

    /// CSV header.
    pub fn csv_header() -> Vec<&'static str> {
        vec![
            "node_id",
            "region",
            "t_window_start",
            "t_window_end",
            "cpu_util",
            "mem_gb",
            "power_w",
            "rf_power_density",
            "duty_cycle",
            "r_energy",
            "r_rf",
            "r_heat",
            "r_sigma",
            "vt",
            "k_score",
            "e_score",
            "r_score",
            "lane",
            "evidence_hex",
        ]
    }

    /// Convert to CSV record.
    pub fn to_csv_record(&self) -> Vec<String> {
        vec![
            self.node_id.clone(),
            self.region.clone(),
            self.t_window_start.clone(),
            self.t_window_end.clone(),
            self.cpu_util.to_string(),
            self.mem_gb.to_string(),
            self.power_w.to_string(),
            self.rf_power_density.to_string(),
            self.duty_cycle.to_string(),
            self.r_energy.to_string(),
            self.r_rf.to_string(),
            self.r_heat.to_string(),
            self.r_sigma.to_string(),
            self.vt.to_string(),
            self.k_score.to_string(),
            self.e_score.to_string(),
            self.r_score.to_string(),
            self.lane.clone(),
            self.evidence_hex.clone(),
        ]
    }
}

/// Errors during shard operations.
#[derive(Error, Debug)]
pub enum ShardError {
    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Writer for storage node shards.
pub struct StorageNodeWriter {
    writer: csv::Writer<std::fs::File>,
}

impl StorageNodeWriter {
    pub fn new(path: &str) -> Result<Self, ShardError> {
        let file_exists = std::path::Path::new(path).exists();
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;

        let mut writer = csv::Writer::from_writer(file);
        if !file_exists {
            writer.write_record(StorageNodeShard::csv_header())?;
        }

        Ok(Self { writer })
    }

    pub fn write(&mut self, shard: &StorageNodeShard) -> Result<(), ShardError> {
        self.writer.write_record(shard.to_csv_record())?;
        self.writer.flush()?;
        Ok(())
    }
}

/// Writer for compute node shards.
pub struct ComputeNodeWriter {
    writer: csv::Writer<std::fs::File>,
}

impl ComputeNodeWriter {
    pub fn new(path: &str) -> Result<Self, ShardError> {
        let file_exists = std::path::Path::new(path).exists();
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;

        let mut writer = csv::Writer::from_writer(file);
        if !file_exists {
            writer.write_record(ComputeNodeShard::csv_header())?;
        }

        Ok(Self { writer })
    }

    pub fn write(&mut self, shard: &ComputeNodeShard) -> Result<(), ShardError> {
        self.writer.write_record(shard.to_csv_record())?;
        self.writer.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_node_safe() {
        let shard = StorageNodeShard::new(
            "node_1".to_string(),
            "rack_a".to_string(),
            "us-west".to_string(),
            45.0,
            -122.0,
            10_000_000_000_000,
            5_000_000_000_000,
            10000,
            300.0,
            28.0,
            24,
            "PROD",
            "abc123",
        );

        assert!(shard.is_safe_for_writes());
        assert!(shard.r_score <= 0.13);
    }

    #[test]
    fn test_compute_node_safe() {
        let shard = ComputeNodeShard::new(
            "build_node_1".to_string(),
            "us-east".to_string(),
            0.3,
            64,
            400.0,
            0.5,
            0.5,
            "PROD",
            "def456",
        );

        assert!(shard.is_safe_for_builds());
    }
}
