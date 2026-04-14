//! Dual-profile Lyapunov residual computation.
//! - Profile 1: Low-latency, stack-allocated kernels for real-time control.
//! - Profile 2: High-throughput, streaming kernels for atlas recomputation.

use ecosafety_core::RiskCoord;
use crate::CorridorSet;

/// Fixed-size residual for high-frequency control (stack allocated).
#[derive(Debug, Clone)]
pub struct ResidualFast<const N: usize> {
    pub r: [f32; N],
    pub w: [f32; N],
    pub c: [f32; N],
    pub vt: f32,
}

impl<const N: usize> ResidualFast<N> {
    #[inline]
    pub fn new(r: [f32; N], w: [f32; N]) -> Self {
        let mut res = Self { r, w, c: [0.0; N], vt: 0.0 };
        res.recompute_vt();
        res
    }

    #[inline]
    pub fn recompute_vt(&mut self) {
        self.vt = 0.0;
        for i in 0..N {
            self.c[i] = self.w[i] * self.r[i] * self.r[i];
            self.vt += self.c[i];
        }
    }

    #[inline]
    pub fn apply_delta(&mut self, changed: &[(usize, f32)]) {
        for &(idx, new_r) in changed {
            debug_assert!(idx < N);
            let old_c = self.c[idx];
            let new_c = self.w[idx] * new_r * new_r;
            self.vt = self.vt - old_c + new_c;
            self.c[idx] = new_c;
            self.r[idx] = new_r;
        }
    }
}

/// Streaming residual for atlas replay (C-compatible layout).
/// Processes shards in batches for SIMD optimization.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct ResidualAtlas {
    pub r: [f32; 7],
    pub w: [f32; 7],
    pub vt: f32,
}

impl ResidualAtlas {
    pub fn from_corridors(corridors: &CorridorSet) -> Self {
        Self {
            r: [0.0; 7],
            w: corridors.weights(),
            vt: 0.0,
        }
    }

    /// Update from a single shard row (raw telemetry).
    /// Designed for streaming over millions of rows.
    #[inline]
    pub fn update_from_raw(&mut self, raw: &[f32; 7], corridordors: &CorridorSet) {
        for i in 0..7 {
            self.r[i] = corridordors.bands[i].normalize(raw[i]).0;
        }
        self.recompute_vt();
    }

    /// Batch update for SIMD processing.
    pub fn update_batch(&mut self, raw_batch: &[[f32; 7]], corridordors: &CorridorSet) -> Vec<f32> {
        raw_batch.iter().map(|raw| {
            self.update_from_raw(raw, corridordors);
            self.vt
        }).collect()
    }

    #[inline]
    fn recompute_vt(&mut self) {
        self.vt = 0.0;
        for i in 0..7 {
            self.vt += self.w[i] * self.r[i] * self.r[i];
        }
    }
}

/// Dual residual (Vt + Ut) for uncertainty tracking.
#[derive(Debug, Clone)]
pub struct DualResidual {
    pub vt: f32,
    pub ut: f32,
    pub r_risk: [f32; 7],
    pub r_uncertainty: [f32; 2], // rsigma, rcalib
    w_risk: [f32; 7],
    w_uncertainty: [f32; 2],
}

impl DualResidual {
    pub fn new(w_risk: [f32; 7], w_uncertainty: [f32; 2]) -> Self {
        Self {
            vt: 0.0, ut: 0.0,
            r_risk: [0.0; 7],
            r_uncertainty: [0.0; 2],
            w_risk,
            w_uncertainty,
        }
    }

    pub fn update_risk(&mut self, r: &[f32; 7]) {
        self.r_risk.copy_from_slice(r);
        self.vt = self.r_risk.iter().zip(&self.w_risk)
            .map(|(r, w)| w * r * r).sum();
    }

    pub fn update_uncertainty(&mut self, rsigma: f32, rcalib: f32) {
        self.r_uncertainty = [rsigma, rcalib];
        self.ut = self.r_uncertainty.iter().zip(&self.w_uncertainty)
            .map(|(r, w)| w * r * r).sum();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fast_vs_atlas_equivalence() {
        let corridors = CorridorSet::default();
        let raw: [f32; 7] = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
        let normalized: Vec<f32> = (0..7).map(|i| corridors.bands[i].normalize(raw[i]).0).collect();
        let r: [f32; 7] = normalized.try_into().unwrap();
        let w = corridors.weights();

        let mut fast = ResidualFast::new(r, w);
        let mut atlas = ResidualAtlas::from_corridors(&corridors);
        atlas.update_from_raw(&raw, &corridors);

        assert!((fast.vt - atlas.vt).abs() < 1e-6);
    }
}
