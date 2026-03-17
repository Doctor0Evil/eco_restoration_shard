// Minimal participation loop: measure, score, write shard, decide.

#![forbid(unsafe_code)]

mod resource_corridors;
mod shard_verify;
mod identity;

use resource_corridors::{phoenix_default_corridors, Residual, RiskCoord, safe_step};
use shard_verify::EcoNetTerminalCorridorShard;

pub struct AgentConfig {
    pub identity: identity::IdentityConfig,
    pub region: String,
}

pub struct Agent {
    pub cfg: AgentConfig,
    pub corridors: Vec<resource_corridors::CorridorBands>,
    pub last_residual: Option<Residual>,
}

impl Agent {
    pub fn new_phoenix() -> Self {
        Self {
            cfg: AgentConfig {
                identity: identity::IdentityConfig::phoenix_default(),
                region: "Phoenix-AZ-US".to_string(),
            },
            corridors: phoenix_default_corridors(),
            last_residual: None,
        }
    }

    pub fn measure_host(&self) -> Vec<RiskCoord> {
        // Placeholder: wire to OS metrics; values must be 0–1 fractions.
        let cpu_frac = 0.05;
        let ram_frac = 0.08;
        let net_frac = 0.02;

        let mut coords = Vec::new();
        for c in &self.corridors {
            let val = match c.var_id {
                "r_cpu" => cpu_frac,
                "r_ram" => ram_frac,
                "r_net" => net_frac,
                _ => 0.0,
            };
            coords.push(RiskCoord {
                value: val,
                bands: c.clone(),
            });
        }
        coords
    }

    pub fn step_once(&mut self) -> resource_corridors::CorridorDecision {
        let coords = self.measure_host();
        let mut current = Residual { vt: 0.0, coords };
        current.recompute();

        let decision = if let Some(prev) = &self.last_residual {
            safe_step(prev, &current)
        } else {
            resource_corridors::CorridorDecision::Ok
        };

        self.last_residual = Some(current);
        decision
    }

    pub fn to_corridor_shard(&self) -> Option<EcoNetTerminalCorridorShard> {
        let res = self.last_residual.as_ref()?;
        let header = shard_verify::ShardHeader {
            shard_type: "EcoNetTerminalCorridors2026v1".to_string(),
            region: self.cfg.region.clone(),
            timestamputc: "2026-03-16T17:15:00Z".to_string(),
            did_author: self.cfg.identity.primary_bostrom.clone(),
            did_signature_hex: "0xa1b2c3d4e5f67890".to_string(),
        };
        let rows = res
            .coords
            .iter()
            .map(|c| shard_verify::TerminalCorridorRow {
                var_id: c.bands.var_id.to_string(),
                measured_frac: c.value,
                safe: c.bands.safe,
                gold: c.bands.gold,
                hard: c.bands.hard,
                weight: c.bands.weight,
                lyap_channel: c.bands.lyap_channel,
            })
            .collect();

        Some(EcoNetTerminalCorridorShard {
            header,
            rows,
            knowledge_factor: 0.94,
            eco_impact_value: 0.90,
            risk_of_harm: 0.13,
        })
    }
}
