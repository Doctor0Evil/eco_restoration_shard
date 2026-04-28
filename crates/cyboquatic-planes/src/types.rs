// File: crates/cyboquatic-planes/src/types.rs

use serde::{Deserialize, Serialize};

/// Normalized [0,1] risk coordinate, clamped.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct RiskCoord(pub f64);

impl RiskCoord {
    pub fn new_clamped(v: f64) -> Self {
        Self(v.max(0.0).min(1.0))
    }

    pub fn value(self) -> f64 {
        self.0
    }
}

/// Unified risk vector including Cyboquatic eco planes.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct RiskVector {
    pub r_energy: RiskCoord,
    pub r_hydraulics: RiskCoord,
    pub r_biology: RiskCoord,
    pub r_carbon: RiskCoord,
    pub r_materials: RiskCoord,
    pub r_biodiversity: RiskCoord,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct LyapunovWeights {
    pub w_energy: f64,
    pub w_hydraulics: f64,
    pub w_biology: f64,
    pub w_carbon: f64,
    pub w_materials: f64,
    pub w_biodiversity: f64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Residual {
    pub value: f64,
}

impl Residual {
    pub fn new(value: f64) -> Self {
        Self { value }
    }
}

impl RiskVector {
    /// V_t = Σ_j w_j r_j^2 — same invariant as your ecosafety core.
    pub fn residual(self, w: LyapunovWeights) -> Residual {
        let mut vt = 0.0;
        vt += w.w_energy * self.r_energy.value().powi(2);
        vt += w.w_hydraulics * self.r_hydraulics.value().powi(2);
        vt += w.w_biology * self.r_biology.value().powi(2);
        vt += w.w_carbon * self.r_carbon.value().powi(2);
        vt += w.w_materials * self.r_materials.value().powi(2);
        vt += w.w_biodiversity * self.r_biodiversity.value().powi(2);
        Residual::new(vt)
    }

    /// Hard-band guard: any coordinate at/above 1.0 is an immediate violation.
    pub fn any_hard_breach(self) -> bool {
        let coords = [
            self.r_energy,
            self.r_hydraulics,
            self.r_biology,
            self.r_carbon,
            self.r_materials,
            self.r_biodiversity,
        ];
        coords.iter().any(|c| c.value() >= 1.0 - 1e-9)
    }
}
