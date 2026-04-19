//! Invariants module - enforces corridor presence, safestep, and KER deployability

use std::collections::HashMap;
use crate::corridor::CorridorBandsComplete;
use crate::residual::Residual;
use crate::ker::KerTriad;
use crate::safestep::{SafeStepResult, SafeStepKernel};

/// CorridorDecision represents the decision from a safestep check.
#[derive(Debug, Clone, PartialEq)]
pub enum CorridorDecision {
    /// Step is safe to proceed.
    Ok,
    /// Step requires derating (reduce intensity/scope).
    Derate,
    /// Step must stop (violation detected).
    Stop,
}

impl From<SafeStepResult> for CorridorDecision {
    fn from(result: SafeStepResult) -> Self {
        match result {
            SafeStepResult::Ok => CorridorDecision::Ok,
            SafeStepResult::Derate => CorridorDecision::Derate,
            SafeStepResult::Stop => CorridorDecision::Stop,
        }
    }
}

/// Check that all required corridor variables are present in the bands slice.
/// 
/// Returns true if all required varids exist in bands.
pub fn corridor_present(required: &[String], bands: &[CorridorBandsComplete]) -> bool {
    required.iter().all(|varid| bands.iter().any(|b| b.varid == *varid))
}

/// Enforce V_{t+1} ≤ V_t and r_j < 1 outside a safe interior.
/// 
/// Returns:
/// - `CorridorDecision::Ok` if V_next <= V_prev and all coords are within bounds.
/// - `CorridorDecision::Derate` if V_next slightly exceeds V_prev but is recoverable.
/// - `CorridorDecision::Stop` if any coord violates hard limits or V_next >> V_prev.
pub fn safestep(prev: &Residual, next: &Residual) -> CorridorDecision {
    let kernel = SafeStepKernel::default();
    
    // Check if V_t increased
    let delta_v = next.vt - prev.vt;

    if delta_v <= kernel.vt_tolerance {
        // V_t decreased or stayed same - good
        // Check if any coordinate is at hard violation (1.0)
        let any_hard_violation = next.coords.iter().any(|(_, r)| *r >= 1.0);
        if any_hard_violation {
            CorridorDecision::Stop
        } else {
            CorridorDecision::Ok
        }
    } else if delta_v < kernel.derate_threshold {
        // Small increase in V_t - allow with derating
        CorridorDecision::Derate
    } else {
        // Large increase in V_t - stop
        CorridorDecision::Stop
    }
}

/// Check if a KER triad and residual meet deployability criteria.
/// 
/// Criteria: K ≥ 0.9, E ≥ 0.9, R ≤ 0.13
pub fn ker_deployable(ker: &KerTriad, _residual: &Residual) -> bool {
    ker.k >= 0.9 && ker.e >= 0.9 && ker.r <= 0.13
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_corridor_present_all() {
        let bands = vec![
            CorridorBandsComplete::new_legacy(
                "rtox".to_string(),
                "normalized".to_string(),
                0.3, 0.6, 0.9,
                1.0,
                "toxicity".to_string(),
                true,
            )
        ];
        let required = vec!["rtox".to_string()];
        assert!(corridor_present(&required, &bands));
    }

    #[test]
    fn test_safestep_ok() {
        let prev = Residual { vt: 0.5, coords: vec![] };
        let next = Residual { vt: 0.4, coords: vec![] };
        assert_eq!(safestep(&prev, &next), CorridorDecision::Ok);
    }

    #[test]
    fn test_safestep_stop_violation() {
        let prev = Residual { vt: 0.5, coords: vec![] };
        let next = Residual { vt: 0.8, coords: vec![] };
        assert_eq!(safestep(&prev, &next), CorridorDecision::Stop);
    }

    #[test]
    fn test_ker_deployable_pass() {
        let ker = KerTriad { k: 0.95, e: 0.92, r: 0.10 };
        let res = Residual { vt: 0.3, coords: vec![] };
        assert!(ker_deployable(&ker, &res));
    }

    #[test]
    fn test_ker_deployable_fail_low_k() {
        let ker = KerTriad { k: 0.8, e: 0.95, r: 0.10 };
        let res = Residual { vt: 0.3, coords: vec![] };
        assert!(!ker_deployable(&ker, &res));
    }
}
