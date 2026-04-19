// File: crates/cyboquatic-energy-mass/src/lib.rs

#![forbid(unsafe_code)]

use cyboquatic_ecosafety_core::{RiskCoord, RiskVector};

#[derive(Clone, Copy, Debug)]
pub struct Sample {
    pub cin_mg_l: f64,
    pub cout_mg_l: f64,
    pub flow_m3_s: f64,
    pub dt_s: f64,
    pub energy_j: f64,
}

/// CEIM mass removed for pollutant x over window.
pub fn mass_removed_kg(samples: &[Sample]) -> f64 {
    const RHO_WATER_KG_M3: f64 = 1000.0;
    let mut m_kg = 0.0;
    for s in samples {
        let dC_mg_l = (s.cin_mg_l - s.cout_mg_l).max(0.0);
        let q_l_s = s.flow_m3_s * 1000.0;
        let mass_mg = dC_mg_l * q_l_s * s.dt_s;
        m_kg += mass_mg / 1.0e6 * RHO_WATER_KG_M3;
    }
    m_kg
}

pub fn energy_total_j(samples: &[Sample]) -> f64 {
    samples.iter().map(|s| s.energy_j).sum()
}

/// Specific energy J/kg removed, with guard.
pub fn specific_energy_j_per_kg(samples: &[Sample]) -> Option<f64> {
    let m = mass_removed_kg(samples);
    if m <= 0.0 {
        return None;
    }
    Some(energy_total_j(samples) / m)
}

/// Normalize specific energy into r_energy plane via corridor bands.
pub fn normalize_specific_energy(
    j_per_kg: f64,
    safe: f64,
    gold: f64,
    hard: f64,
) -> RiskCoord {
    let x = if j_per_kg <= safe {
        0.0
    } else if j_per_kg <= gold {
        (j_per_kg - safe) / (gold - safe) * 0.5
    } else if j_per_kg <= hard {
        0.5 + (j_per_kg - gold) / (hard - gold) * 0.5
    } else {
        1.0
    };
    RiskCoord::clamped(x)
}

/// Embed the specific‑energy coordinate into RiskVector.
pub fn embed_energy_plane(
    rv: &mut RiskVector,
    r_energy: RiskCoord,
) {
    rv.r_energy = r_energy;
}
