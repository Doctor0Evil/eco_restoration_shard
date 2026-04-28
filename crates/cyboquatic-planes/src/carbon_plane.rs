// File: crates/cyboquatic-planes/src/carbon_plane.rs

use serde::{Deserialize, Serialize};
use crate::types::RiskCoord;

/// Raw carbon–energy accounting over a window (cycle, hour, etc.).
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct CarbonRaw {
    /// Total mass of CO₂e processed (kg), sign independent.
    pub mass_processed_kg: f64,
    /// Net sequestered CO₂e (kg). Positive = removed from atmosphere.
    pub net_sequestered_kg: f64,
    /// Electrical/mechanical energy used (kWh).
    pub energy_kwh: f64,
}

/// Corridor parameters for net carbon performance (kg CO₂e / kWh).
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct CarbonCorridor {
    /// Strongly negative "safe" target (e.g. -0.30 kg CO₂e/kWh).
    pub safe_kg_per_kwh: f64,
    /// Near-neutral "gold" band (e.g. -0.05 kg CO₂e/kWh).
    pub gold_kg_per_kwh: f64,
    /// Worst acceptable "hard" limit (e.g. +0.05 kg CO₂e/kWh).
    pub hard_kg_per_kwh: f64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct CarbonScore {
    pub r_carbon: RiskCoord,
    /// Effective net intensity including grid mix (kg CO₂e/kWh).
    pub intensity_kg_per_kwh: f64,
    /// True if within hard corridor band.
    pub corridor_ok: bool,
}

impl CarbonCorridor {
    /// Normalize emissions intensity into r_carbon ∈ [0,1].
    ///
    /// safe < gold < hard; strongly negative intensities map near 0,
    /// hard or worse map near 1.0. Monotone in harmful direction.
    pub fn score(self, raw: CarbonRaw, grid_emissions_kg_per_kwh: f64) -> CarbonScore {
        let intensity = if raw.energy_kwh <= 0.0 {
            // Unknown / suspicious — pessimistically near hard.
            self.hard_kg_per_kwh
        } else {
            let gross_intensity = -raw.net_sequestered_kg / raw.energy_kwh;
            gross_intensity + grid_emissions_kg_per_kwh
        };

        let lo = self.safe_kg_per_kwh;
        let hi = self.hard_kg_per_kwh;
        let mut r = if intensity <= lo {
            0.0
        } else if intensity >= hi {
            1.0
        } else {
            (intensity - lo) / (hi - lo)
        };
        r = r.max(0.0).min(1.0);

        let corridor_ok = intensity <= self.hard_kg_per_kwh + 1e-9;

        CarbonScore {
            r_carbon: RiskCoord::new_clamped(r),
            intensity_kg_per_kwh: intensity,
            corridor_ok,
        }
    }
}
