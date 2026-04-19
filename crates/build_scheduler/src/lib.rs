//! build_scheduler — Ecosafety scheduler for Rust builds and agent workloads
//!
//! This crate reads `ComputeNodeShard` data and only starts heavy builds
//! (e.g., full workspace compile) when r_energy, r_rf, and r_heat are in
//! safe/gold bands and V_{t+1} will not increase.

#![forbid(unsafe_code)]

use cyboquatic_ecosafety_core::{KerWindow, LyapunovWeights, Residual, RiskCoord, RiskVector};
use storage_shards::ComputeNodeShard;
use thiserror::Error;
use tracing::{debug, info, warn};

/// Build job request.
#[derive(Clone, Debug)]
pub struct BuildJob {
    pub job_id: String,
    pub job_type: BuildType,
    pub estimated_power_w: f64,
    pub estimated_duration_min: u32,
}

#[derive(Clone, Debug)]
pub enum BuildType {
    WorkspaceCompile,
    CrateCompile(String),
    TestSuite,
    IndexBuild,
}

/// Scheduler decision.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScheduleDecision {
    /// Build can start now.
    StartNow,
    /// Defer until conditions improve.
    Defer,
    /// Reject due to ecosafety constraints.
    Reject,
}

/// Errors during scheduling.
#[derive(Error, Debug)]
pub enum SchedulerError {
    #[error("node not safe: r_score={0}")]
    NodeNotSafe(f64),

    #[error("Vt would increase: {0} -> {1}")]
    VtIncrease(f64, f64),
}

/// Build scheduler that evaluates compute node shards.
pub struct BuildScheduler {
    weights: LyapunovWeights,
    vt_epsilon: f64,
}

impl BuildScheduler {
    pub fn new() -> Self {
        Self {
            weights: LyapunovWeights::default_carbon_negative(),
            vt_epsilon: 1e-6,
        }
    }

    /// Evaluate if a build can start on a node.
    pub fn evaluate_build(
        &self,
        node: &ComputeNodeShard,
        job: &BuildJob,
        vt_prev: Residual,
    ) -> Result<(ScheduleDecision, Option<Residual>), SchedulerError> {
        // Check node safety first
        if !node.is_safe_for_builds() {
            warn!(
                "Node {} not safe for builds: r_score={}",
                node.node_id, node.r_score
            );
            return Ok((ScheduleDecision::Defer, None));
        }

        // Estimate impact of build on energy plane
        let estimated_r_energy = self.estimate_r_energy(job.estimated_power_w);

        // Build projected RiskVector
        let rv_next = RiskVector {
            energy: RiskCoord::new(estimated_r_energy),
            hydraulics: RiskCoord::new(0.0),
            biology: RiskCoord::new(0.0),
            carbon: RiskCoord::new(0.0),
            materials: RiskCoord::new(0.0),
            biodiversity: RiskCoord::new(0.0),
            sigma: RiskCoord::new(node.r_sigma),
        };

        let vt_next = cyboquatic_ecosafety_core::residual(&rv_next, &self.weights);

        // Check Lyapunov invariant
        if vt_next.value > vt_prev.value + self.vt_epsilon {
            warn!(
                "Build would increase Vt: {:.6} -> {:.6}",
                vt_prev.value, vt_next.value
            );
            return Ok((ScheduleDecision::Reject, Some(vt_next)));
        }

        info!(
            "Build {} approved on node {} (Vt: {:.6} -> {:.6})",
            job.job_id, node.node_id, vt_prev.value, vt_next.value
        );

        Ok((ScheduleDecision::StartNow, Some(vt_next)))
    }

    /// Estimate r_energy from power draw.
    fn estimate_r_energy(&self, power_w: f64) -> f64 {
        use cyboquatic_ecosafety_core::CorridorBands;
        let bands = CorridorBands::new(0.0, 500.0, 2000.0);
        bands.normalize(power_w).value()
    }

    /// Create a KER window summary for a completed build.
    pub fn build_ker_summary(
        &self,
        node: &ComputeNodeShard,
        success: bool,
    ) -> KerWindow {
        let r = if success {
            node.r_score * 0.9 // Slight improvement on success
        } else {
            node.r_score * 1.1 // Slight degradation on failure
        }
        .clamp(0.0, 1.0);

        let k = if success { 0.95 } else { 0.85 };
        let e = (1.0 - r).clamp(0.0, 1.0);

        KerWindow { k, e, r }
    }
}

impl Default for BuildScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schedule_safe_build() {
        let scheduler = BuildScheduler::new();
        let node = ComputeNodeShard::new(
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

        let job = BuildJob {
            job_id: "test_compile".to_string(),
            job_type: BuildType::WorkspaceCompile,
            estimated_power_w: 500.0,
            estimated_duration_min: 30,
        };

        let vt_prev = Residual { value: 0.5 };
        let (decision, vt_next) = scheduler.evaluate_build(&node, &job, vt_prev).unwrap();

        assert_eq!(decision, ScheduleDecision::StartNow);
        assert!(vt_next.is_some());
        assert!(vt_next.unwrap().value <= vt_prev.value);
    }

    #[test]
    fn test_schedule_unsafe_build() {
        let scheduler = BuildScheduler::new();
        let node = ComputeNodeShard::new(
            "build_node_hot".to_string(),
            "us-west".to_string(),
            0.9, // High CPU util
            128,
            1800.0, // High power
            8.0,    // High RF
            0.95,
            "PROD",
            "ghi789",
        );

        let job = BuildJob {
            job_id: "heavy_compile".to_string(),
            job_type: BuildType::WorkspaceCompile,
            estimated_power_w: 1500.0,
            estimated_duration_min: 60,
        };

        let vt_prev = Residual { value: 0.3 };
        let (decision, _) = scheduler.evaluate_build(&node, &job, vt_prev).unwrap();

        // Should defer because node is not safe
        assert_eq!(decision, ScheduleDecision::Defer);
    }
}
