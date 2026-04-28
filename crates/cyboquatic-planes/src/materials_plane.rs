// File: crates/cyboquatic-planes/src/materials_plane.rs

use serde::{Deserialize, Serialize};
use crate::types::RiskCoord;

/// Kinetic and hazard parameters for a biodegradable substrate.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct MaterialKinetics {
    /// Time to 90% mass loss (days) under reference conditions.
    pub t90_days: f64,
    /// Normalized ecotoxicity score [0,1] (higher = worse).
    pub r_tox: f64,
    /// Normalized micro-residue risk [0,1].
    pub r_micro: f64,
    /// Normalized leachate CEC risk [0,1].
    pub r_leach_cec: f64,
    /// Normalized PFAS-like residual risk [0,1].
    pub r_pfas_resid: f64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct MaterialsCorridors {
    /// Max acceptable t90 for "safe" band (days).
    pub t90_safe_days: f64,
    /// Max acceptable t90 for "hard" band (days).
    pub t90_hard_days: f64,
    /// Weights for composite aggregation.
    pub w_t90: f64,
    pub w_tox: f64,
    pub w_micro: f64,
    pub w_leach_cec: f64,
    pub w_pfas_resid: f64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct MaterialsScore {
    pub r_materials: RiskCoord,
    pub r_t90: RiskCoord,
    pub corridor_ok: bool,
}

impl MaterialsCorridors {
    fn normalize_t90(&self, t90_days: f64) -> RiskCoord {
        let lo = self.t90_safe_days;
        let hi = self.t90_hard_days;
        let r = if t90_days <= lo {
            0.0
        } else if t90_days >= hi {
            1.0
        } else {
            (t90_days - lo) / (hi - lo)
        };
        RiskCoord::new_clamped(r)
    }

    pub fn score(self, kin: MaterialKinetics) -> MaterialsScore {
        let r_t90 = self.normalize_t90(kin.t90_days);

        let w_sum =
            self.w_t90
            + self.w_tox
            + self.w_micro
            + self.w_leach_cec
            + self.w_pfas_resid;

        let (w_t90, w_tox, w_micro, w_leach, w_pfas) = if w_sum > 0.0 {
            (
                self.w_t90 / w_sum,
                self.w_tox / w_sum,
                self.w_micro / w_sum,
                self.w_leach_cec / w_sum,
                self.w_pfas_resid / w_sum,
            )
        } else {
            (0.2, 0.2, 0.2, 0.2, 0.2)
        };

        let r_materials_sq =
            w_t90 * r_t90.value().powi(2) +
            w_tox * kin.r_tox.powi(2) +
            w_micro * kin.r_micro.powi(2) +
            w_leach * kin.r_leach_cec.powi(2) +
            w_pfas * kin.r_pfas_resid.powi(2);

        let r_materials = RiskCoord::new_clamped(r_materials_sq.sqrt());

        let corridor_ok = kin.t90_days <= self.t90_hard_days + 1e-9
            && kin.r_tox <= 1.0 + 1e-9
            && kin.r_micro <= 1.0 + 1e-9
            && kin.r_leach_cec <= 1.0 + 1e-9
            && kin.r_pfas_resid <= 1.0 + 1e-9;

        MaterialsScore {
            r_materials,
            r_t90,
            corridor_ok,
        }
    }
}
