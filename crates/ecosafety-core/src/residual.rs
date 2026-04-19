// crates/ecosafety-core/src/residual.rs
//! Five-layer residual kernel with stack-allocated Lyapunov computation.
//! 
//! This module implements the shared residual engine so all layers
//! (hydraulics/FOG, materials, biodiversity, carbon, data quality)
//! use one frozen grammar (r_j, w_j, V_t).

use core::marker::PhantomData;

/// Stack-allocated residual state with cached per-coordinate contributions.
/// 
/// The const generic N represents the number of risk coordinates
/// (typically 5 for the five planes: hydraulics, materials, biodiversity, carbon, data-quality).
#[derive(Clone, Copy, Debug)]
pub struct ResidualState<const N: usize> {
    /// Normalized risk coordinates r_j ∈ [0,1]
    pub r: [f64; N],
    /// Weights from CorridorBands
    pub w: [f64; N],
    /// Cached contributions c_j = w_j * r_j^2
    pub c: [f64; N],
    /// Lyapunov residual V_t = Σ c_j
    pub vt: f64,
}

impl<const N: usize> ResidualState<N> {
    /// Create a new zero-initialized residual state.
    pub fn new() -> Self {
        Self {
            r: [0.0; N],
            w: [0.0; N],
            c: [0.0; N],
            vt: 0.0,
        }
    }

    /// Create a residual state from raw arrays.
    /// 
    /// Caller must ensure weights are non-negative and risk coords are in [0,1].
    pub fn from_arrays(r: [f64; N], w: [f64; N]) -> Self {
        let mut state = Self {
            r,
            w,
            c: [0.0; N],
            vt: 0.0,
        };
        state.recompute_vt();
        state
    }

    /// Recompute all cached contributions and V_t from scratch.
    /// 
    /// Use this when multiple coordinates change or after initialization.
    #[inline]
    pub fn recompute_vt(&mut self) {
        let mut acc = 0.0;
        for j in 0..N {
            let rj = self.r[j];
            let wj = self.w[j];
            let cj = wj * rj * rj;
            self.c[j] = cj;
            acc += cj;
        }
        self.vt = acc;
    }

    /// Apply a delta update to a single coordinate in O(1) time.
    /// 
    /// Updates r[idx], c[idx], and vt incrementally.
    /// 
    /// # Panics
    /// Panics if idx >= N.
    #[inline]
    pub fn apply_delta(&mut self, idx: usize, new_r: f64) {
        assert!(idx < N, "index {} out of bounds for ResidualState<{}>", idx, N);
        let old_c = self.c[idx];
        let w = self.w[idx];
        let new_c = w * new_r * new_r;
        self.r[idx] = new_r;
        self.c[idx] = new_c;
        self.vt = self.vt - old_c + new_c;
    }

    /// Set weight for coordinate idx.
    /// 
    /// After changing weights, call `recompute_vt()` to update cached values.
    #[inline]
    pub fn set_weight(&mut self, idx: usize, w: f64) {
        assert!(idx < N, "index {} out of bounds for ResidualState<{}>", idx, N);
        self.w[idx] = w;
    }

    /// Get the current Lyapunov residual V_t.
    #[inline]
    pub fn vt(&self) -> f64 {
        self.vt
    }

    /// Get risk coordinate at index.
    #[inline]
    pub fn r(&self, idx: usize) -> f64 {
        assert!(idx < N, "index {} out of bounds for ResidualState<{}>", idx, N);
        self.r[idx]
    }

    /// Get weight at index.
    #[inline]
    pub fn w(&self, idx: usize) -> f64 {
        assert!(idx < N, "index {} out of bounds for ResidualState<{}>", idx, N);
        self.w[idx]
    }

    /// Check if all risk coordinates are within bounds [0, 1].
    #[inline]
    pub fn all_coords_bounded(&self) -> bool {
        self.r.iter().all(|&rj| (0.0..=1.0).contains(&rj))
    }

    /// Check if V_t is non-negative (should always be true for valid weights).
    #[inline]
    pub fn vt_non_negative(&self) -> bool {
        self.vt >= 0.0
    }
}

impl<const N: usize> Default for ResidualState<N> {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a safe step check.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SafeStepResult {
    /// Step is safe: all r_j <= 1 and V_{t+1} <= V_t (within tolerance).
    Ok,
    /// One or more risk coordinates exceed 1.0.
    RiskCoordinateExceeded { index: usize, value: f64 },
    /// Lyapunov residual increased beyond tolerance.
    ResidualIncreased { prev_vt: f64, next_vt: f64 },
}

/// Error types for residual operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResidualError {
    /// Risk coordinate out of [0,1] range.
    RiskCoordOutOfBounds { index: usize, value: f64 },
    /// Weight is negative.
    NegativeWeight { index: usize, value: f64 },
    /// Index out of bounds.
    IndexOutOfBounds { index: usize, size: usize },
}

impl core::fmt::Display for ResidualError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ResidualError::RiskCoordOutOfBounds { index, value } => {
                write!(f, "risk coordinate[{}] = {} is out of [0,1]", index, value)
            }
            ResidualError::NegativeWeight { index, value } => {
                write!(f, "weight[{}] = {} is negative", index, value)
            }
            ResidualError::IndexOutOfBounds { index, size } => {
                write!(f, "index {} out of bounds for size {}", index, size)
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ResidualError {}

/// Configuration for the safestep kernel.
#[derive(Clone, Copy, Debug)]
pub struct SafeStepConfig {
    /// Tolerance for risk coordinate bound checks (default: 1e-9).
    pub risk_tolerance: f64,
    /// Tolerance for Lyapunov residual increase (default: 1e-9).
    pub vt_tolerance: f64,
}

impl Default for SafeStepConfig {
    fn default() -> Self {
        Self {
            risk_tolerance: 1e-9,
            vt_tolerance: 1e-9,
        }
    }
}

/// Execute the safestep kernel.
/// 
/// Enforces:
/// - All r_j <= 1 + risk_tolerance
/// - V_{t+1} <= V_t + vt_tolerance
/// 
/// Returns `SafeStepResult::Ok` if the step is safe, otherwise indicates
/// which invariant was violated.
#[inline]
pub fn safestep<const N: usize>(
    prev: &ResidualState<N>,
    next: &ResidualState<N>,
    config: SafeStepConfig,
) -> SafeStepResult {
    // Check all risk coordinates are bounded
    for (idx, &rj) in next.r.iter().enumerate() {
        if rj > 1.0 + config.risk_tolerance {
            return SafeStepResult::RiskCoordinateExceeded {
                index: idx,
                value: rj,
            };
        }
    }

    // Check Lyapunov residual does not increase beyond tolerance
    if next.vt > prev.vt + config.vt_tolerance {
        return SafeStepResult::ResidualIncreased {
            prev_vt: prev.vt,
            next_vt: next.vt,
        };
    }

    SafeStepResult::Ok
}

/// Validate a residual state has proper structure.
/// 
/// Checks:
/// - All risk coordinates in [0, 1]
/// - All weights non-negative
/// - V_t is non-negative
/// - Cached contributions match recomputed values (within tolerance)
pub fn validate_residual<const N: usize>(
    state: &ResidualState<N>,
    tolerance: f64,
) -> Result<(), ResidualError> {
    for (idx, &rj) in state.r.iter().enumerate() {
        if rj < 0.0 || rj > 1.0 {
            return Err(ResidualError::RiskCoordOutOfBounds { index: idx, value: rj });
        }
    }

    for (idx, &wj) in state.w.iter().enumerate() {
        if wj < 0.0 {
            return Err(ResidualError::NegativeWeight { index: idx, value: wj });
        }
    }

    if state.vt < 0.0 {
        return Err(ResidualError::RiskCoordOutOfBounds { index: 0, value: state.vt });
    }

    // Verify cached contributions
    let mut computed_vt = 0.0;
    for idx in 0..N {
        let expected_c = state.w[idx] * state.r[idx] * state.r[idx];
        if (state.c[idx] - expected_c).abs() > tolerance {
            return Err(ResidualError::RiskCoordOutOfBounds {
                index: idx,
                value: state.c[idx],
            });
        }
        computed_vt += expected_c;
    }

    if (state.vt - computed_vt).abs() > tolerance {
        return Err(ResidualError::RiskCoordOutOfBounds {
            index: N,
            value: state.vt,
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_residual_state_new() {
        let state: ResidualState<5> = ResidualState::new();
        assert_eq!(state.vt(), 0.0);
        assert!(state.allcoords_bounded());
        assert!(state.vt_non_negative());
    }

    #[test]
    fn test_residual_state_from_arrays() {
        let r = [0.5, 0.3, 0.8, 0.2, 0.6];
        let w = [1.0, 1.0, 1.0, 1.0, 1.0];
        let state = ResidualState::from_arrays(r, w);
        
        // V_t = 0.25 + 0.09 + 0.64 + 0.04 + 0.36 = 1.38
        let expected_vt = 0.25 + 0.09 + 0.64 + 0.04 + 0.36;
        assert!((state.vt() - expected_vt).abs() < 1e-10);
    }

    #[test]
    fn test_apply_delta() {
        let r = [0.5, 0.3, 0.8];
        let w = [1.0, 2.0, 0.5];
        let mut state = ResidualState::from_arrays(r, w);
        let prev_vt = state.vt();
        
        // Change r[1] from 0.3 to 0.4
        // Old c[1] = 2.0 * 0.09 = 0.18
        // New c[1] = 2.0 * 0.16 = 0.32
        // Delta = 0.14
        state.apply_delta(1, 0.4);
        
        assert_eq!(state.r(1), 0.4);
        assert!((state.vt() - (prev_vt + 0.14)).abs() < 1e-10);
    }

    #[test]
    fn test_safestep_ok() {
        let r_prev = [0.3, 0.4, 0.5];
        let w = [1.0, 1.0, 1.0];
        let prev = ResidualState::from_arrays(r_prev, w);
        
        let r_next = [0.2, 0.3, 0.4];
        let next = ResidualState::from_arrays(r_next, w);
        
        let result = safestep(&prev, &next, SafeStepConfig::default());
        assert_eq!(result, SafeStepResult::Ok);
    }

    #[test]
    fn test_safestep_risk_exceeded() {
        let r_prev = [0.3, 0.4, 0.5];
        let w = [1.0, 1.0, 1.0];
        let prev = ResidualState::from_arrays(r_prev, w);
        
        let r_next = [0.2, 1.5, 0.4]; // r[1] > 1
        let next = ResidualState::from_arrays(r_next, w);
        
        let result = safestep(&prev, &next, SafeStepConfig::default());
        assert!(matches!(result, SafeStepResult::RiskCoordinateExceeded { index: 1, .. }));
    }

    #[test]
    fn test_safestep_residual_increased() {
        let r_prev = [0.2, 0.2, 0.2];
        let w = [1.0, 1.0, 1.0];
        let prev = ResidualState::from_arrays(r_prev, w);
        
        let r_next = [0.5, 0.5, 0.5]; // Higher V_t
        let next = ResidualState::from_arrays(r_next, w);
        
        let result = safestep(&prev, &next, SafeStepConfig::default());
        assert!(matches!(result, SafeStepResult::ResidualIncreased { .. }));
    }

    #[test]
    fn test_validate_residual() {
        let r = [0.5, 0.3, 0.8];
        let w = [1.0, 2.0, 0.5];
        let state = ResidualState::from_arrays(r, w);
        
        assert!(validate_residual(&state, 1e-10).is_ok());
    }

    #[test]
    fn test_validate_residual_bad_coord() {
        let mut state: ResidualState<3> = ResidualState::new();
        state.r[0] = 1.5; // Out of bounds
        state.w = [1.0, 1.0, 1.0];
        state.recompute_vt();
        
        assert!(matches!(
            validate_residual(&state, 1e-10),
            Err(ResidualError::RiskCoordOutOfBounds { .. })
        ));
    }

    #[test]
    fn test_validate_residual_negative_weight() {
        let r = [0.5, 0.3, 0.8];
        let w = [1.0, -0.5, 0.5]; // Negative weight
        let state = ResidualState::from_arrays(r, w);
        
        assert!(matches!(
            validate_residual(&state, 1e-10),
            Err(ResidualError::NegativeWeight { .. })
        ));
    }
}
