//! SafeStep kernel - enforces Lyapunov safety invariants
//! 
//! This module implements the core safestep decision kernel that ensures:
//! - All risk coordinates r_j ≤ 1.0 (hard bound)
//! - V_{t+1} ≤ V_t + ε (Lyapunov non-increase with tolerance)
//! - Monotone corridor normalization

use crate::residual::ResidualState;

/// Result of a safe step evaluation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SafeStepResult {
    /// Step is safe: V_next ≤ V_prev and all r_j < 1.0
    Ok,
    /// Step requires derating: small V increase but recoverable
    Derate,
    /// Step must stop: hard violation detected
    Stop,
}

/// Error types for safe step validation
#[derive(Debug, Clone, PartialEq)]
pub enum SafeStepError {
    /// Risk coordinate exceeds hard bound (r_j > 1.0)
    RiskBoundViolation { index: usize, value: f64 },
    /// Residual increased beyond tolerance
    ResidualIncrease { delta: f64, threshold: f64 },
    /// Invalid state (e.g., NaN or Inf values)
    InvalidState { description: String },
}

/// SafeStepKernel configuration with tolerances
#[derive(Debug, Clone, Copy)]
pub struct SafeStepKernel {
    /// Tolerance for V_t increase (ε_v)
    pub vt_tolerance: f64,
    /// Tolerance for r_j bound (ε_r), allows r_j ≤ 1.0 + ε_r
    pub r_bound_tolerance: f64,
    /// Derate threshold: if ΔV < derate_threshold, allow with derating
    pub derate_threshold: f64,
}

impl Default for SafeStepKernel {
    fn default() -> Self {
        Self {
            vt_tolerance: 1e-9,       // Numeric precision tolerance
            r_bound_tolerance: 0.0,   // Strict bound enforcement
            derate_threshold: 0.1,    // 10% V_t increase allowed with derating
        }
    }
}

impl SafeStepKernel {
    /// Create a new SafeStepKernel with custom tolerances
    pub fn new(vt_tolerance: f64, r_bound_tolerance: f64, derate_threshold: f64) -> Self {
        Self {
            vt_tolerance,
            r_bound_tolerance,
            derate_threshold,
        }
    }
    
    /// Validate a single ResidualState for internal consistency
    pub fn validate_state<const N: usize>(&self, state: &ResidualState<N>) -> Result<(), SafeStepError> {
        // Check for NaN/Inf
        for (i, &r) in state.r.iter().enumerate() {
            if r.is_nan() || r.is_infinite() {
                return Err(SafeStepError::InvalidState {
                    description: format!("Risk coordinate {} is NaN or Inf", i),
                });
            }
        }
        if state.vt.is_nan() || state.vt.is_infinite() {
            return Err(SafeStepError::InvalidState {
                description: "V_t is NaN or Inf".to_string(),
            });
        }
        Ok(())
    }
    
    /// Check if all risk coordinates are within bounds
    pub fn check_bounds<const N: usize>(&self, state: &ResidualState<N>) -> Result<(), SafeStepError> {
        let bound = 1.0 + self.r_bound_tolerance;
        for (i, &r) in state.r.iter().enumerate() {
            if r > bound {
                return Err(SafeStepError::RiskBoundViolation {
                    index: i,
                    value: r,
                });
            }
        }
        Ok(())
    }
    
    /// Evaluate a safe step from previous to next state
    /// 
    /// Returns:
    /// - `SafeStepResult::Ok` if V_next ≤ V_prev + ε and all bounds satisfied
    /// - `SafeStepResult::Derate` if small V increase but within derate threshold
    /// - `SafeStepResult::Stop` if hard violation detected
    pub fn evaluate<const N: usize>(
        &self,
        prev: &ResidualState<N>,
        next: &ResidualState<N>,
    ) -> Result<SafeStepResult, SafeStepError> {
        // Validate both states
        self.validate_state(prev)?;
        self.validate_state(next)?;
        
        // Check bounds on next state
        if let Err(e) = self.check_bounds(next) {
            return match e {
                SafeStepError::RiskBoundViolation { .. } => Ok(SafeStepResult::Stop),
                other => Err(other),
            };
        }
        
        // Compute ΔV = V_next - V_prev
        let delta_v = next.vt - prev.vt;
        
        if delta_v <= self.vt_tolerance {
            // V decreased or stayed same (within tolerance) - OK
            Ok(SafeStepResult::Ok)
        } else if delta_v <= self.derate_threshold {
            // Small increase - allow with derating
            Ok(SafeStepResult::Derate)
        } else {
            // Large increase - stop
            Ok(SafeStepResult::Stop)
        }
    }
    
    /// Convenience method: check if a transition is safe (returns bool)
    pub fn is_safe<const N: usize>(
        &self,
        prev: &ResidualState<N>,
        next: &ResidualState<N>,
    ) -> bool {
        matches!(self.evaluate(prev, next), Ok(SafeStepResult::Ok))
    }
    
    /// Apply a proposed delta and check if it would be safe
    /// 
    /// This is useful for "what-if" analysis before committing changes.
    pub fn check_delta<const N: usize>(
        &self,
        current: &ResidualState<N>,
        idx: usize,
        new_r: f64,
    ) -> Result<SafeStepResult, SafeStepError> {
        if idx >= N {
            return Err(SafeStepError::InvalidState {
                description: format!("Index {} out of bounds for ResidualState<{}>", idx, N),
            });
        }
        
        // Clone and apply delta
        let mut next = current.clone();
        next.apply_delta(idx, new_r);
        
        self.evaluate(current, &next)
    }
}

/// Legacy function for backward compatibility
pub fn safestep<const N: usize>(prev: &ResidualState<N>, next: &ResidualState<N>) -> SafeStepResult {
    let kernel = SafeStepKernel::default();
    kernel.evaluate(prev, next).unwrap_or(SafeStepResult::Stop)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safestep_ok() {
        let kernel = SafeStepKernel::default();
        let mut prev: ResidualState<3> = ResidualState::new();
        prev.set_all([0.5, 0.3, 0.2]);
        
        let mut next: ResidualState<3> = ResidualState::new();
        next.set_all([0.4, 0.3, 0.2]); // V decreased
        
        assert_eq!(kernel.evaluate(&prev, &next).unwrap(), SafeStepResult::Ok);
    }
    
    #[test]
    fn test_safestep_derate() {
        let kernel = SafeStepKernel::default();
        let mut prev: ResidualState<3> = ResidualState::new();
        prev.set_all([0.3, 0.3, 0.3]);
        
        let mut next: ResidualState<3> = ResidualState::new();
        next.set_all([0.4, 0.3, 0.3]); // Small V increase
        
        assert_eq!(kernel.evaluate(&prev, &next).unwrap(), SafeStepResult::Derate);
    }
    
    #[test]
    fn test_safestep_stop_large_increase() {
        let kernel = SafeStepKernel::default();
        let mut prev: ResidualState<3> = ResidualState::new();
        prev.set_all([0.2, 0.2, 0.2]);
        
        let mut next: ResidualState<3> = ResidualState::new();
        next.set_all([0.8, 0.8, 0.8]); // Large V increase
        
        assert_eq!(kernel.evaluate(&prev, &next).unwrap(), SafeStepResult::Stop);
    }
    
    #[test]
    fn test_safestep_stop_bound_violation() {
        let kernel = SafeStepKernel::default();
        let mut prev: ResidualState<3> = ResidualState::new();
        prev.set_all([0.3, 0.3, 0.3]);
        
        let mut next: ResidualState<3> = ResidualState::new();
        next.set_all([1.5, 0.3, 0.3]); // Bound violation
        
        assert_eq!(kernel.evaluate(&prev, &next).unwrap(), SafeStepResult::Stop);
    }
    
    #[test]
    fn test_check_delta() {
        let kernel = SafeStepKernel::default();
        let mut current: ResidualState<3> = ResidualState::new();
        current.set_all([0.3, 0.3, 0.3]);
        
        // Safe delta
        assert_eq!(kernel.check_delta(&current, 0, 0.4).unwrap(), SafeStepResult::Derate);
        
        // Unsafe delta (bound violation)
        assert_eq!(kernel.check_delta(&current, 0, 1.5).unwrap(), SafeStepResult::Stop);
    }
    
    #[test]
    fn test_validate_invalid_state() {
        let kernel = SafeStepKernel::default();
        let mut invalid: ResidualState<3> = ResidualState::new();
        invalid.r[0] = f64::NAN;
        invalid.recompute_vt();
        
        assert!(kernel.validate_state(&invalid).is_err());
    }
    
    #[test]
    fn test_custom_tolerances() {
        // Stricter kernel
        let strict = SafeStepKernel::new(1e-12, 0.0, 0.01);
        let mut prev: ResidualState<2> = ResidualState::new();
        prev.set_all([0.5, 0.5]);
        
        let mut next: ResidualState<2> = ResidualState::new();
        next.set_all([0.51, 0.5]); // Very small increase
        
        // With strict derate threshold, this should be Stop
        assert_eq!(strict.evaluate(&prev, &next).unwrap(), SafeStepResult::Stop);
        
        // With default kernel, this might be Derate
        let default = SafeStepKernel::default();
        let result = default.evaluate(&prev, &next).unwrap();
        assert!(result == SafeStepResult::Derate || result == SafeStepResult::Ok);
    }
}
