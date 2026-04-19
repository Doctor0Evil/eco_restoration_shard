//! Biodegradable substrate shards - kinetics and ecosafety data.

use serde::{Deserialize, Serialize};
use ecosafety_core::KerTriad;

/// SubstrateKineticsShard holds lab and sim data for biodegradable substrates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubstrateKineticsShard {
    pub substrate_id: String,
    pub channel_type: String,
    pub environment_profile: String,
    pub protocol_id: String, // ISO/OECD reference
    
    // Physical kinetics
    pub t_half_d: f64,       // Half-life in days
    pub t90_d: f64,          // Time to 90% degradation in days
    pub mass_loss_28d: f64,  // Mass loss at 28 days (%)
    pub mass_loss_180d: f64, // Mass loss at 180 days (%)
    
    // Ecosafety hooks
    pub lcms_analytes_json: String,           // JSON array of LCMS analytes
    pub leachate_concentrations_json: String, // JSON map of leachate concentrations
    pub microplastics_counts: u64,            // Microplastics count
}

impl SubstrateKineticsShard {
    /// Create a new kinetics shard from raw lab data.
    pub fn from_lab_data(
        substrate_id: &str,
        channel_type: &str,
        environment_profile: &str,
        protocol_id: &str,
        t_half_d: f64,
        t90_d: f64,
        mass_loss_28d: f64,
        mass_loss_180d: f64,
    ) -> Self {
        SubstrateKineticsShard {
            substrate_id: substrate_id.to_string(),
            channel_type: channel_type.to_string(),
            environment_profile: environment_profile.to_string(),
            protocol_id: protocol_id.to_string(),
            t_half_d,
            t90_d,
            mass_loss_28d,
            mass_loss_180d,
            lcms_analytes_json: "[]".to_string(),
            leachate_concentrations_json: "{}".to_string(),
            microplastics_counts: 0,
        }
    }
}

/// SubstrateEcosafetyShard holds deployability envelope for biodegradable substrates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubstrateEcosafetyShard {
    pub substrate_id: String,
    pub channel_type: String,
    pub environment_profile: String,
    pub protocol_id: String,
    
    // Normalized coordinates (rx values in [0,1])
    pub rx_rbiodeg_speed: f64,
    pub rx_rresidual_mass: f64,
    pub rx_rmicroplastics: f64,
    pub rx_racutetox: f64,
    pub rx_rchronictox: f64,
    
    // Residual and KER
    pub vt: f64,
    pub knowledge_factor: f64,
    pub eco_impact: f64,
    pub risk_of_harm: f64,
    pub ker_deployable: bool,
}

impl SubstrateEcosafetyShard {
    /// Compute V_t from normalized coordinates (simplified Lyapunov residual).
    pub fn compute_vt(&mut self) {
        self.vt = self.rx_rbiodeg_speed.powi(2)
            + self.rx_rresidual_mass.powi(2)
            + self.rx_rmicroplastics.powi(2)
            + self.rx_racutetox.powi(2)
            + self.rx_rchronictox.powi(2);
    }

    /// Recompute KER triad and deployability.
    pub fn recompute_ker(&mut self) -> KerTriad {
        self.compute_vt();

        let ker = KerTriad {
            k: self.knowledge_factor,
            e: self.eco_impact,
            r: self.risk_of_harm,
        };

        // Deployability: K ≥ 0.9, E ≥ 0.9, R ≤ 0.13
        self.ker_deployable = ker.k >= 0.9 && ker.e >= 0.9 && ker.r <= 0.13;

        ker
    }

    /// Check if this substrate meets production requirements.
    /// 
    /// Invariant: any substrate used in a production recipe must have
    /// ker_deployable=true and all toxicity/microplastics coordinates below gold bands.
    pub fn is_production_ready(&self) -> bool {
        if !self.ker_deployable {
            return false;
        }

        // Check that toxicity and microplastics are below threshold (e.g., < 0.5 = below gold)
        self.rx_racutetox < 0.5
            && self.rx_rchronictox < 0.5
            && self.rx_rmicroplastics < 0.5
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kinetics_shard_creation() {
        let shard = SubstrateKineticsShard::from_lab_data(
            "substrate_001",
            "compost",
            "aerobic_25C",
            "ISO14851",
            30.0,
            90.0,
            60.0,
            95.0,
        );

        assert_eq!(shard.substrate_id, "substrate_001");
        assert_eq!(shard.t_half_d, 30.0);
    }

    #[test]
    fn test_ecosafety_shard_deployable() {
        let mut shard = SubstrateEcosafetyShard {
            substrate_id: "substrate_001".to_string(),
            channel_type: "compost".to_string(),
            environment_profile: "aerobic_25C".to_string(),
            protocol_id: "ISO14851".to_string(),
            rx_rbiodeg_speed: 0.2,
            rx_rresidual_mass: 0.1,
            rx_rmicroplastics: 0.05,
            rx_racutetox: 0.08,
            rx_rchronictox: 0.06,
            vt: 0.0,
            knowledge_factor: 0.95,
            eco_impact: 0.92,
            risk_of_harm: 0.10,
            ker_deployable: false,
        };

        let ker = shard.recompute_ker();
        assert!(ker.k >= 0.9);
        assert!(shard.ker_deployable);
        assert!(shard.is_production_ready());
    }

    #[test]
    fn test_ecosafety_shard_not_ready_high_tox() {
        let mut shard = SubstrateEcosafetyShard {
            substrate_id: "substrate_002".to_string(),
            channel_type: "compost".to_string(),
            environment_profile: "aerobic_25C".to_string(),
            protocol_id: "ISO14851".to_string(),
            rx_rbiodeg_speed: 0.2,
            rx_rresidual_mass: 0.1,
            rx_rmicroplastics: 0.05,
            rx_racutetox: 0.7, // High acute toxicity
            rx_rchronictox: 0.06,
            vt: 0.0,
            knowledge_factor: 0.95,
            eco_impact: 0.92,
            risk_of_harm: 0.10,
            ker_deployable: false,
        };

        shard.recompute_ker();
        assert!(shard.ker_deployable); // KER passes
        assert!(!shard.is_production_ready()); // But tox is too high
    }
}
