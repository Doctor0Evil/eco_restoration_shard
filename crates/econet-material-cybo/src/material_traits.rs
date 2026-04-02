#![no_std]

use cyboquatic_ecosafety_core::RiskCoord;

pub struct MaterialKinetics {
    pub t90_days:        f32,
    pub r_tox:           RiskCoord,
    pub r_micro:         RiskCoord,
    pub caloric_density: f32,   // MJ/kg
}

pub struct MaterialRisks {
    pub r_degrade: RiskCoord,
    pub r_tox:     RiskCoord,
    pub r_micro:   RiskCoord,
}

pub trait AntSafeSubstrate {
    fn kinetics(&self) -> MaterialKinetics;
    fn risks(&self) -> MaterialRisks;

    fn corridor_ok(&self) -> bool {
        let k = self.kinetics();
        let r = self.risks();

        let t90_hard_days: f32 = 180.0;
        let t90_gold_days: f32 = 120.0;

        let r_degrade = if k.t90_days <= t90_gold_days {
            0.0
        } else if k.t90_days >= t90_hard_days {
            1.0
        } else {
            (k.t90_days - t90_gold_days) / (t90_hard_days - t90_gold_days)
        };

        // Hard gates derived from Phoenix 2026 material corridors
        let ok_tox   = r.r_tox   <= 0.10;
        let ok_micro = r.r_micro <= 0.05;
        let ok_cal   = k.caloric_density <= 0.30;
        let ok_deg   = r_degrade <= 1.0;

        ok_tox && ok_micro && ok_cal && ok_deg
    }
}

pub trait CyboNodeCompatible {
    fn introduces_conflicting_contaminants(&self) -> bool;

    fn node_ok(&self) -> bool {
        !self.introduces_conflicting_contaminants()
    }
}
