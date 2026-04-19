//! Ecosafety core library with five-layer residual kernel and corridor grammar.

pub mod residual;
pub mod corridor;

// Re-export main types for convenience
pub use residual::{ResidualState, SafeStepResult, SafeStepConfig, safestep, validate_residual, ResidualError};
pub use corridor::{
    CorridorBands, CorridorBandsComplete, CorridorBandsBuilder,
    RiskCoord, CorridorTable, NormalizationError, normalize_measurement,
};

// Legacy re-exports for backward compatibility
#[derive(Clone, Copy, Debug)]
pub struct LegacyCorridorBands {
    pub var_id: &'static str,
    pub units: &'static str,
    pub safe: f64,
    pub gold: f64,
    pub hard: f64,
    pub weight: f64,
    pub lyap_channel: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CorridorDecision {
    Ok,
    Derate,
    Stop,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResidualCheck {
    Ok,
    ViolatedAxis,
    IncreasedResidual,
}

#[derive(Clone, Debug)]
pub struct MetricFields {
    pub k: f64,
    pub e: f64,
    pub r: f64,
    pub rx: Vec<f64>,
    pub vt: f64,
}

impl MetricFields {
    pub fn is_well_formed(&self) -> bool {
        self.vt >= 0.0 && self.rx.iter().all(|&r| (0.0..=1.0).contains(&r))
    }
}

/// Legacy normalization function (deprecated, use corridor::normalize_measurement)
#[deprecated(note = "Use corridor::normalize_measurement instead")]
pub fn normalize_metric(x: f64, bands: &LegacyCorridorBands) -> RiskCoord {
    let r = if x <= bands.safe {
        0.0
    } else if x <= bands.gold {
        0.5 * (x - bands.safe) / (bands.gold - bands.safe)
    } else if x <= bands.hard {
        0.5 + 0.5 * (x - bands.gold) / (bands.hard - bands.gold)
    } else {
        1.0
    };
    RiskCoord::new(r.clamp(0.0, 1.0), 0.01)
}

/// Legacy safe step function (deprecated, use residual::safestep)
#[deprecated(note = "Use residual::safestep instead")]
pub fn safe_step(prev_vt: f64, next_vt: f64, tolerance: f64) -> CorridorDecision {
    if next_vt <= prev_vt + tolerance {
        CorridorDecision::Ok
    } else {
        CorridorDecision::Derate
    }
}

/// Legacy residual check (deprecated, use residual::validate_residual)
#[deprecated(note = "Use residual::validate_residual instead")]
pub fn residual_ok(_prev: &MetricFields, next: &MetricFields) -> ResidualCheck {
    if next.is_well_formed() {
        ResidualCheck::Ok
    } else {
        ResidualCheck::ViolatedAxis
    }
}
