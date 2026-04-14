//! SafeStepGate: the only path from ecosafety decisions to physical actuation.
//! Enforces corridorpresent, residualsafe, kerdeployable, and routevariant invariants.

use ecosafety_core::{RiskCoord, EvidenceHex, Lane};
use crate::{CorridorSet, ResidualState, QPUShardV1};

/// Route variant for actuation decision.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RouteVariant {
    /// Full deployment allowed.
    Deploy,
    /// Reduced capacity operation.
    Derate(f32), // derating factor 0.0-1.0
    /// Halt all actuation, observe only.
    Stop,
    /// Passive observation (no actuation attempted).
    Observe,
}

/// SafeStepGate configuration from ALN contract.
#[derive(Debug, Clone)]
pub struct SafeStepConfig {
    pub v_max: f32,
    pub u_max: f32,
    pub v_trend_window: usize,
    pub v_trend_threshold: f32,
}

impl Default for SafeStepConfig {
    fn default() -> Self {
        Self {
            v_max: 0.3,
            u_max: 0.4,
            v_trend_window: 10,
            v_trend_threshold: 0.01,
        }
    }
}

/// The SafeStepGate trait: any actuator controller must implement this.
pub trait SafeStepGate {
    /// Evaluate whether a proposed action is safe.
    /// Returns RouteVariant based on current state and invariants.
    fn evaluate(
        &self,
        residual: &ResidualState,
        corridors: &CorridorSet,
        recent_shards: &[QPUShardV1],
        lane: Lane,
    ) -> RouteVariant;

    /// Check invariants and produce a bounded command if safe.
    /// This is the ONLY function that may emit a physical command.
    fn step(
        &mut self,
        residual: &ResidualState,
        corridors: &CorridorSet,
        recent_shards: &[QPUShardV1],
        lane: Lane,
        requested_action: f32, // normalized 0-1 request
    ) -> Option<f32> {
        match self.evaluate(residual, corridors, recent_shards, lane) {
            RouteVariant::Deploy => Some(requested_action.clamp(0.0, 1.0)),
            RouteVariant::Derate(factor) => Some(requested_action * factor),
            RouteVariant::Stop | RouteVariant::Observe => None,
        }
    }
}

/// Standard SafeStepGate implementation.
pub struct StandardSafeStepGate {
    config: SafeStepConfig,
    v_history: Vec<f32>,
}

impl StandardSafeStepGate {
    pub fn new(config: SafeStepConfig) -> Self {
        Self {
            config,
            v_history: Vec::with_capacity(config.v_trend_window),
        }
    }

    fn check_corridor_present(&self, corridors: &CorridorSet) -> bool {
        corridors.validate().is_ok()
    }

    fn check_residual_safe(&self, residual: &ResidualState) -> bool {
        residual.vt <= self.config.v_max && residual.ut <= self.config.u_max
    }

    fn check_ker_deployable(&self, shard: &QPUShardV1, lane: Lane) -> bool {
        match lane {
            Lane::RESEARCH => true,
            Lane::PILOT => shard.ker_k >= 0.80 && shard.ker_e >= 0.75 && shard.ker_r <= 0.20,
            Lane::PROD => shard.ker_k >= 0.90 && shard.ker_e >= 0.90 && shard.ker_r <= 0.13,
        }
    }

    fn check_vt_non_increase(&self) -> bool {
        if self.v_history.len() < 2 {
            return true;
        }
        let recent_avg: f32 = self.v_history.iter().sum::<f32>() / self.v_history.len() as f32;
        let older_avg = if self.v_history.len() >= self.config.v_trend_window {
            let older: Vec<f32> = self.v_history.iter()
                .take(self.config.v_trend_window).copied().collect();
            older.iter().sum::<f32>() / older.len() as f32
        } else {
            self.v_history[0]
        };
        (recent_avg - older_avg) <= self.config.v_trend_threshold
    }

    fn update_history(&mut self, vt: f32) {
        self.v_history.push(vt);
        if self.v_history.len() > self.config.v_trend_window * 2 {
            self.v_history.remove(0);
        }
    }
}

impl SafeStepGate for StandardSafeStepGate {
    fn evaluate(
        &self,
        residual: &ResidualState,
        corridors: &CorridorSet,
        recent_shards: &[QPUShardV1],
        lane: Lane,
    ) -> RouteVariant {
        // 1. Corridor presence check
        if !self.check_corridor_present(corridors) {
            return RouteVariant::Stop;
        }

        // 2. Residual safety
        if !self.check_residual_safe(residual) {
            if residual.vt > self.config.v_max * 1.5 {
                return RouteVariant::Stop;
            }
            return RouteVariant::Derate(0.3);
        }

        // 3. KER deployable (use most recent shard)
        if let Some(latest) = recent_shards.last() {
            if !self.check_ker_deployable(latest, lane) {
                return RouteVariant::Derate(0.5);
            }
        }

        // 4. Vt non-increase trend
        if !self.check_vt_non_increase() {
            return RouteVariant::Derate(0.7);
        }

        RouteVariant::Deploy
    }

    fn step(
        &mut self,
        residual: &ResidualState,
        corridors: &CorridorSet,
        recent_shards: &[QPUShardV1],
        lane: Lane,
        requested_action: f32,
    ) -> Option<f32> {
        self.update_history(residual.vt);
        let route = self.evaluate(residual, corridors, recent_shards, lane);
        match route {
            RouteVariant::Deploy => Some(requested_action.clamp(0.0, 1.0)),
            RouteVariant::Derate(factor) => Some(requested_action * factor),
            RouteVariant::Stop | RouteVariant::Observe => None,
        }
    }
}

/// Pilot-Gate: determines when a node can advance lanes.
pub struct PilotGate {
    pub rules: PilotGateRules,
    pub observation_start: std::time::SystemTime,
    pub shard_count: usize,
}

#[derive(Debug, Clone)]
pub struct PilotGateRules {
    pub min_ker_k: f32,
    pub min_ker_e: f32,
    pub max_ker_r: f32,
    pub max_vt: f32,
    pub observation_days: u32,
    pub required_shards: u32,
}

impl PilotGate {
    pub fn for_lane(lane: Lane) -> Self {
        let rules = match lane {
            Lane::RESEARCH => PilotGateRules {
                min_ker_k: 0.0, min_ker_e: 0.0, max_ker_r: 1.0, max_vt: 0.8,
                observation_days: 1, required_shards: 10,
            },
            Lane::PILOT => PilotGateRules {
                min_ker_k: 0.80, min_ker_e: 0.75, max_ker_r: 0.20, max_vt: 0.5,
                observation_days: 30, required_shards: 1000,
            },
            Lane::PROD => PilotGateRules {
                min_ker_k: 0.90, min_ker_e: 0.90, max_ker_r: 0.13, max_vt: 0.3,
                observation_days: 90, required_shards: 10000,
            },
        };
        Self {
            rules,
            observation_start: std::time::SystemTime::now(),
            shard_count: 0,
        }
    }

    pub fn record_shard(&mut self, shard: &QPUShardV1) {
        self.shard_count += 1;
    }

    pub fn eligible_for_promotion(&self, current_lane: Lane) -> Option<Lane> {
        let elapsed = std::time::SystemTime::now()
            .duration_since(self.observation_start)
            .unwrap_or_default();
        let days_elapsed = elapsed.as_secs() / 86400;

        if self.shard_count as u32 >= self.rules.required_shards
            && days_elapsed as u32 >= self.rules.observation_days {
            match current_lane {
                Lane::RESEARCH => Some(Lane::PILOT),
                Lane::PILOT => Some(Lane::PROD),
                Lane::PROD => None,
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safestep_blocks_high_vt() {
        let config = SafeStepConfig::default();
        let mut gate = StandardSafeStepGate::new(config);
        let residual = ResidualState { vt: 0.5, ut: 0.1, c: vec![] };
        let corridors = CorridorSet::default();
        let shards = vec![];
        let result = gate.step(&residual, &corridors, &shards, Lane::PROD, 0.8);
        assert_eq!(result, None); // v_max=0.3, so should Stop -> None
    }
}
