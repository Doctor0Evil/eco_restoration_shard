// File: crates/cyboquatic-eco-planes/src/lib.rs

use cyboquatic_ecosafety_core::{RiskCoord, RiskVector};

/// Normalization parameters for carbon plane.
pub struct CarbonNorm {
    pub co2eq_neg_min: f64, // strong sequestration (kg CO2e / cycle)
    pub co2eq_gold_max: f64, // near-neutral band upper bound
    pub co2eq_hard_max: f64, // worst acceptable emissions
}

impl CarbonNorm {
    /// Map net CO2e per cycle → r_carbon ∈ [0,1].
    pub fn normalize(&self, net_co2eq_kg: f64) -> RiskCoord {
        if net_co2eq_kg <= self.co2eq_neg_min {
            return RiskCoord::clamped(0.0);
        }
        if net_co2eq_kg <= self.co2eq_gold_max {
            // map [neg_min, gold_max] into [0, 0.3]
            let span = self.co2eq_gold_max - self.co2eq_neg_min;
            let rel = (net_co2eq_kg - self.co2eq_neg_min) / span.max(1e-9);
            return RiskCoord::clamped(0.3 * rel);
        }
        // map [gold_max, hard_max] into [0.3, 1.0]
        let span = (self.co2eq_hard_max - self.co2eq_gold_max).max(1e-9);
        let rel = (net_co2eq_kg - self.co2eq_gold_max) / span;
        let r = 0.3 + 0.7 * rel;
        RiskCoord::clamped(r)
    }
}

/// Normalization for biodiversity: higher habitat quality → lower risk.
pub struct BiodiversityNorm {
    pub habitat_index_min: f64, // worst habitat score
    pub habitat_index_gold: f64, // good habitat score
    pub habitat_index_max: f64, // best achievable
}

impl BiodiversityNorm {
    /// Map habitat index → r_biodiversity ∈ [0,1].
    pub fn normalize(&self, habitat_index: f64) -> RiskCoord {
        if habitat_index >= self.habitat_index_max {
            return RiskCoord::clamped(0.0);
        }
        if habitat_index >= self.habitat_index_gold {
            // map [gold, max] into [0, 0.3]
            let span = (self.habitat_index_max - self.habitat_index_gold).max(1e-9);
            let rel = (self.habitat_index_max - habitat_index) / span;
            return RiskCoord::clamped(0.3 * rel);
        }
        // map [min, gold) into (0.3, 1.0]
        let span = (self.habitat_index_gold - self.habitat_index_min).max(1e-9);
        let rel = (habitat_index - self.habitat_index_min) / span;
        let r = 0.3 + 0.7 * (1.0 - rel);
        RiskCoord::clamped(r)
    }
}

/// Helper to update carbon/biodiversity planes inside a RiskVector.
pub fn with_updated_eco_planes(
    mut rv: RiskVector,
    r_carbon: RiskCoord,
    r_biodiversity: RiskCoord,
) -> RiskVector {
    rv.r_carbon = r_carbon;
    rv.r_biodiversity = r_biodiversity;
    rv
}
