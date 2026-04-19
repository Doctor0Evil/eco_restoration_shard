//! Residual kernel module - computes V_t residual from normalized coordinates
//! 
//! Implements the five-layer residual kernel with stack-allocated state,
//! cached per-coordinate contributions, and canonical plane order.

use serde::{Deserialize, Serialize};
use crate::corridor::CorridorBandsComplete;

/// ResidualState represents the Lyapunov residual V_t with cached contributions.
/// Stack-allocated for fixed N coordinates with O(1) delta updates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResidualState<const N: usize> {
    /// Normalized risk coordinates r_j ∈ [0,1]
    pub r: [f64; N],
    /// Weights from CorridorBands
    pub w: [f64; N],
    /// Cached contributions c_j = w_j * r_j^2
    pub c: [f64; N],
    /// Lyapunov residual V_t = sum(c_j)
    pub vt: f64,
}

impl<const N: usize> ResidualState<N> {
    /// Create a new ResidualState with zero values
    pub fn new() -> Self {
        Self {
            r: [0.0; N],
            w: [1.0; N], // Default uniform weights
            c: [0.0; N],
            vt: 0.0,
        }
    }
    
    /// Create a new ResidualState with specified weights and recompute V_t
    pub fn with_weights(weights: [f64; N]) -> Self {
        let mut state = Self::new();
        state.w = weights;
        state.recompute_vt();
        state
    }
    
    /// Recompute V_t from all coordinates (O(N))
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
    
    /// Apply a delta update to a single coordinate (O(1))
    /// Updates r[idx], c[idx], and vt atomically
    #[inline]
    pub fn apply_delta(&mut self, idx: usize, new_r: f64) {
        debug_assert!(idx < N, "Index {} out of bounds for ResidualState<{}>", idx, N);
        let old_c = self.c[idx];
        let w = self.w[idx];
        let new_c = w * new_r * new_r;
        self.r[idx] = new_r;
        self.c[idx] = new_c;
        self.vt = self.vt - old_c + new_c;
    }
    
    /// Set all coordinates at once and recompute V_t
    pub fn set_all(&mut self, coords: [f64; N]) {
        self.r = coords;
        self.recompute_vt();
    }
    
    /// Get a specific risk coordinate
    #[inline]
    pub fn get_r(&self, idx: usize) -> f64 {
        debug_assert!(idx < N);
        self.r[idx]
    }
    
    /// Get the cached contribution for a coordinate
    #[inline]
    pub fn get_c(&self, idx: usize) -> f64 {
        debug_assert!(idx < N);
        self.c[idx]
    }
    
    /// Check if all coordinates are within bounds (r_j <= 1.0)
    pub fn all_within_bounds(&self) -> bool {
        self.r.iter().all(|&r| r <= 1.0)
    }
    
    /// Find the maximum risk coordinate
    pub fn max_r(&self) -> (usize, f64) {
        let mut max_idx = 0;
        let mut max_val = self.r[0];
        for (i, &r) in self.r.iter().enumerate() {
            if r > max_val {
                max_val = r;
                max_idx = i;
            }
        }
        (max_idx, max_val)
    }
}

impl<const N: usize> Default for ResidualState<N> {
    fn default() -> Self {
        Self::new()
    }
}

/// Legacy Residual type for backward compatibility (heap-allocated)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Residual {
    pub vt: f64,
    pub coords: Vec<(String, f64)>,
}

/// Compute the residual V_t from normalized coordinates and corridor bands.
/// 
/// V_t = sum_i(w_i * r_i^2) where w_i is the weight from CorridorBands and r_i is the normalized coordinate.
pub fn compute_residual(
    coords: &[(String, f64)],
    bands: &[CorridorBandsComplete],
) -> Residual {
    let mut vt = 0.0;
    let mut normalized_coords = Vec::new();

    for (varid, value) in coords {
        // Find matching band
        let (norm_value, weight) = bands
            .iter()
            .find(|b| b.varid == *varid)
            .map(|b| (b.normalize_coord(*value), b.weight))
            .unwrap_or((1.0, 1.0)); // Conservative default if no band found

        vt += weight * norm_value * norm_value;
        normalized_coords.push((varid.clone(), norm_value));
    }

    Residual {
        vt,
        coords: normalized_coords,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_residual_state_new() {
        let state: ResidualState<5> = ResidualState::new();
        assert_eq!(state.vt, 0.0);
        assert!(state.all_within_bounds());
    }
    
    #[test]
    fn test_residual_state_with_weights() {
        let weights = [0.2, 0.2, 0.2, 0.2, 0.2];
        let mut state = ResidualState::<5>::with_weights(weights);
        assert_eq!(state.vt, 0.0);
        
        state.apply_delta(0, 0.5);
        assert!((state.vt - 0.2 * 0.25).abs() < 1e-10);
    }
    
    #[test]
    fn test_residual_state_apply_delta() {
        let weights = [1.0, 1.0, 1.0];
        let mut state = ResidualState::<3>::with_weights(weights);
        
        state.apply_delta(0, 0.5);
        assert!((state.vt - 0.25).abs() < 1e-10);
        
        state.apply_delta(1, 0.5);
        assert!((state.vt - 0.5).abs() < 1e-10);
        
        state.apply_delta(0, 1.0);
        assert!((state.vt - 1.25).abs() < 1e-10);
    }
    
    #[test]
    fn test_residual_state_recompute() {
        let weights = [0.5, 0.5];
        let mut state = ResidualState::<2>::with_weights(weights);
        state.set_all([0.5, 0.5]);
        
        // V_t = 0.5 * 0.25 + 0.5 * 0.25 = 0.25
        assert!((state.vt - 0.25).abs() < 1e-10);
    }
    
    #[test]
    fn test_residual_state_max_r() {
        let mut state: ResidualState<4> = ResidualState::new();
        state.set_all([0.1, 0.9, 0.3, 0.5]);
        
        let (idx, val) = state.max_r();
        assert_eq!(idx, 1);
        assert_eq!(val, 0.9);
    }
    
    #[test]
    fn test_residual_state_bounds_check() {
        let mut state: ResidualState<3> = ResidualState::new();
        state.set_all([0.5, 0.8, 0.9]);
        assert!(state.all_within_bounds());
        
        state.apply_delta(1, 1.5);
        assert!(!state.all_within_bounds());
    }
}
