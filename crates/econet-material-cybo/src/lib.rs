// File: econet-material-cybo/src/lib.rs

#![forbid(unsafe_code)]

use cyboquatic_ecosafety_core::{CorridorBands, KerTriad, KerWindow, RiskCoord, RiskVector};
use std::fmt;

#[derive(Clone, Copy, Debug)]
pub struct MaterialKinetics {
    pub t90_days: f64,
    pub r_tox: f64,
    pub r_micro: f64,
    pub r_leach_cec: f64,
    pub r_pfas_resid: f64,
    pub caloric_density_mj_per_kg: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct MaterialCorridors {
    pub t90_max_days: f64,
    pub t90_gold_days: f64,
    pub r_tox_gold_max: f64,
    pub r_micro_max: f64,
    pub r_leach_max: f64,
    pub r_pfas_max: f64,
    pub caloric_density_max: f64,
}

impl Default for MaterialCorridors {
    fn default() -> Self {
        MaterialCorridors {
            t90_max_days: 180.0,
            t90_gold_days: 120.0,
            r_tox_gold_max: 0.10,
            r_micro_max: 0.05,
            r_leach_max: 0.10,
            r_pfas_max: 0.10,
            caloric_density_max: 0.30,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MaterialRisks {
    pub r_t90: RiskCoord,
    pub r_tox: RiskCoord,
    pub r_micro: RiskCoord,
    pub r_leach_cec: RiskCoord,
    pub r_pfas_resid: RiskCoord,
}

impl MaterialRisks {
    pub fn from_kinetics(k: &MaterialKinetics, c: &MaterialCorridors) -> Self {
        let t_corr = CorridorBands {
            x_safe: 0.0,
            x_gold: c.t90_gold_days,
            x_hard: c.t90_max_days,
        };
        let r_t90 = t_corr.normalize(k.t90_days);

        let r_tox = RiskCoord::new_clamped(k.r_tox / c.r_tox_gold_max);
        let r_micro = RiskCoord::new_clamped(k.r_micro / c.r_micro_max);
        let r_leach_cec = RiskCoord::new_clamped(k.r_leach_cec / c.r_leach_max);
        let r_pfas_resid = RiskCoord::new_clamped(k.r_pfas_resid / c.r_pfas_max);

        MaterialRisks {
            r_t90,
            r_tox,
            r_micro,
            r_leach_cec,
            r_pfas_resid,
        }
    }

    pub fn to_vector(&self) -> RiskVector {
        RiskVector {
            coords: vec![
                self.r_t90,
                self.r_tox,
                self.r_micro,
                self.r_leach_cec,
                self.r_pfas_resid,
            ],
        }
    }

    pub fn ecoimpact_score(&self, weights: &[f64; 5]) -> f64 {
        let risks = [
            self.r_t90.value(),
            self.r_tox.value(),
            self.r_micro.value(),
            self.r_leach_cec.value(),
            self.r_pfas_resid.value(),
        ];
        let mut s = 0.0;
        let mut wsum = 0.0;
        for (r, w) in risks.iter().zip(weights.iter()) {
            let w = w.max(0.0);
            s += w * r;
            wsum += w;
        }
        if wsum == 0.0 {
            1.0
        } else {
            let r_bar = s / wsum;
            (1.0 - r_bar).max(0.0)
        }
    }

    pub fn ker(&self, weights: &[f64; 5]) -> KerTriad {
        let mut win = KerWindow::new();
        let rv = self.to_vector();
        let e = self.ecoimpact_score(weights);
        let max_r = rv.max().value();
        let lyapunov_safe = true;
        win.update_step(lyapunov_safe, &rv);
        let mut triad = win.finalize();
        triad.e_ecoimpact = e;
        triad.r_risk_of_harm = max_r;
        triad
    }
}

/// Biodegradable, node-compatible substrate traits.

pub trait AntSafeSubstrate {
    fn kinetics(&self) -> &MaterialKinetics;
    fn corridors(&self) -> &MaterialCorridors;

    fn corridor_ok(&self) -> bool {
        let k = self.kinetics();
        let c = self.corridors();
        if k.t90_days > c.t90_max_days {
            return false;
        }
        if k.r_tox > c.r_tox_gold_max {
            return false;
        }
        if k.r_micro > c.r_micro_max {
            return false;
        }
        if k.r_leach_cec > c.r_leach_max {
            return false;
        }
        if k.r_pfas_resid > c.r_pfas_max {
            return false;
        }
        if k.caloric_density_mj_per_kg > c.caloric_density_max {
            return false;
        }
        true
    }
}

pub trait CyboNodeCompatible: AntSafeSubstrate {
    fn introduces_pfas_mass(&self) -> bool;
    fn introduces_nutrient_mass(&self) -> bool;

    fn node_compatible(&self) -> bool {
        self.corridor_ok()
            && !self.introduces_pfas_mass()
            && !self.introduces_nutrient_mass()
    }
}

#[derive(Clone, Debug)]
pub struct SubstrateSpec {
    pub id: String,
    pub kinetics: MaterialKinetics,
    pub corridors: MaterialCorridors,
    pub pfas_back_leach: bool,
    pub nutrient_back_leach: bool,
}

impl AntSafeSubstrate for SubstrateSpec {
    fn kinetics(&self) -> &MaterialKinetics {
        &self.kinetics
    }
    fn corridors(&self) -> &MaterialCorridors {
        &self.corridors
    }
}

impl CyboNodeCompatible for SubstrateSpec {
    fn introduces_pfas_mass(&self) -> bool {
        self.pfas_back_leach
    }
    fn introduces_nutrient_mass(&self) -> bool {
        self.nutrient_back_leach
    }
}

impl SubstrateSpec {
    pub fn risks(&self) -> MaterialRisks {
        MaterialRisks::from_kinetics(&self.kinetics, &self.corridors)
    }

    pub fn ker(&self, weights: &[f64; 5]) -> KerTriad {
        self.risks().ker(weights)
    }

    pub fn deployment_allowed(&self, weights: &[f64; 5]) -> bool {
        if !self.node_compatible() {
            return false;
        }
        let ker = self.ker(weights);
        ker.k_knowledge >= 0.90 && ker.e_ecoimpact >= 0.90 && ker.r_risk_of_harm <= 0.13
    }
}

impl fmt::Display for SubstrateSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let w = [0.2, 0.2, 0.2, 0.2, 0.2];
        let ker = self.ker(&w);
        write!(f, "Substrate {} {}", self.id, ker)
    }
}
