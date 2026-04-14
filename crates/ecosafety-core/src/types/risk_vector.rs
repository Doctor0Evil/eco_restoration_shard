//! Risk vector and Lyapunov residual types with ALN binding.
//! Uses the `#[derive(AlnShard)]` procedural macro to enforce compile-time
//! schema validation and auto-generate normalization/trait implementations.

use ecosafety_macros::AlnShard;
use ecosafety_core::{
    RiskCoord, EvidenceHex, SignatureHex, UnixMillis, NodeId,
    Residual, RiskVector, KerDeployable,
};

/// Canonical risk vector for a Cyboquatic node.
/// Fields correspond exactly to EcoSafetyRiskVector ALN family.
/// The macro ensures:
/// - Every mandatory ALN column has a matching field with correct mapped type.
/// - Normalization functions `normalize_r_*` are generated for each `isriskcoord=true`.
/// - `RiskVector` trait is implemented with canonical field order.
/// - Provenance fields `evidencehex` and `signinghex` are validated.
#[derive(Debug, Clone, AlnShard)]
#[aln_contract(
    family = "EcoSafetyRiskVector",
    version = "2.0.0",
    path = "../ecosafety-specs/grammar/EcoSafetyGrammar2026v1.aln"
)]
pub struct EcoSafetyRiskVectorV2 {
    // Risk coordinates (order matters for canonical serialization)
    pub r_energy: RiskCoord,
    pub r_hydraulic: RiskCoord,
    pub r_biology: RiskCoord,
    pub r_carbon: RiskCoord,
    pub r_materials: RiskCoord,
    pub r_dataquality: RiskCoord,
    pub r_sigma: RiskCoord,

    // Provenance
    pub evidencehex: EvidenceHex,
    pub signinghex: Option<SignatureHex>,

    // Metadata
    pub timestamp: UnixMillis,
    pub node_id: NodeId,
}

/// High-frequency control residual using const generics.
/// Stored entirely on stack for sub-microsecond Vt updates.
#[derive(Debug, Clone)]
pub struct ResidualFixed<const N: usize> {
    /// Normalized risk coordinates r_j ∈ [0,1]
    pub r: [f32; N],
    /// Weights w_j (sum = 1.0)
    pub w: [f32; N],
    /// Cached contributions c_j = w_j * r_j²
    pub c: [f32; N],
    /// Lyapunov residual V_t = Σ c_j
    pub vt: f32,
}

impl<const N: usize> ResidualFixed<N> {
    /// Full recompute from r and w arrays.
    #[inline]
    pub fn recompute_vt(&mut self) {
        self.vt = 0.0;
        for i in 0..N {
            self.c[i] = self.w[i] * self.r[i] * self.r[i];
            self.vt += self.c[i];
        }
    }

    /// Incremental update when a subset of coordinates change.
    /// `changed` is a slice of (index, new_r_value) pairs.
    #[inline]
    pub fn apply_delta(&mut self, changed: &[(usize, f32)]) {
        for &(idx, new_r) in changed {
            debug_assert!(idx < N);
            debug_assert!((0.0..=1.0).contains(&new_r));
            let old_c = self.c[idx];
            let new_c = self.w[idx] * new_r * new_r;
            self.vt = self.vt - old_c + new_c;
            self.c[idx] = new_c;
            self.r[idx] = new_r;
        }
    }
}

/// Dual residual for production lanes with fixed-point arithmetic.
/// Used in CI replays to ensure long-term numerical stability.
#[derive(Debug, Clone)]
pub struct ResidualFixedPoint {
    /// Fixed-point representation (e.g., 16.16 or 24.8 depending on target)
    pub r: [i32; 7],
    pub w: [i32; 7],
    pub c: [i64; 7],
    pub vt: i64,
    scale: u8,
}

impl ResidualFixedPoint {
    /// Convert from floating point with specified scaling factor.
    pub fn from_float(r: &[f32; 7], w: &[f32; 7], scale: u8) -> Self {
        let factor = (1i32 << scale) as f32;
        let mut res = Self {
            r: [0; 7],
            w: [0; 7],
            c: [0; 7],
            vt: 0,
            scale,
        };
        for i in 0..7 {
            res.r[i] = (r[i] * factor).round() as i32;
            res.w[i] = (w[i] * factor).round() as i32;
            res.c[i] = (res.w[i] as i64 * res.r[i] as i64 * res.r[i] as i64) >> (2 * scale);
            res.vt += res.c[i];
        }
        res
    }

    pub fn to_float_vt(&self) -> f32 {
        (self.vt as f32) / ((1i32 << self.scale) as f32).powi(2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn incremental_vs_full_recompute() {
        let r = [0.2, 0.3, 0.1, 0.4, 0.15, 0.05, 0.25];
        let w = [0.12, 0.18, 0.20, 0.15, 0.15, 0.10, 0.10];
        let mut full = ResidualFixed { r, w, c: [0.0; 7], vt: 0.0 };
        full.recompute_vt();

        let mut incr = full.clone();
        // Simulate change in two coordinates
        incr.apply_delta(&[(2, 0.35), (5, 0.08)]);

        // Full recompute with updated values
        let mut r2 = r;
        r2[2] = 0.35;
        r2[5] = 0.08;
        let mut full2 = ResidualFixed { r: r2, w, c: [0.0; 7], vt: 0.0 };
        full2.recompute_vt();

        assert!((incr.vt - full2.vt).abs() < 1e-6);
    }
}
