// crates/ecosafety-core-canal/src/lib.rs
#![forbid(unsafe_code)]
#![no_std]

pub type RiskCoord = f32;
pub type Residual  = f32;

#[derive(Clone, Copy, Debug)]
pub struct CorridorBands {
    pub soft_max: RiskCoord,   // r_soft < 1.0
    pub hard_max: RiskCoord,   // = 1.0 by grammar
}

impl CorridorBands {
    /// Normalize raw x into r_x with r_x = 1 at corridor edge.
    pub fn normalize(&self, raw: f32, raw_soft: f32, raw_hard: f32) -> RiskCoord {
        if raw_hard <= raw_soft { return 1.0; }
        let span = raw_hard - raw_soft;
        let r = if raw <= raw_soft {
            0.0
        } else if raw >= raw_hard {
            1.0
        } else {
            (raw - raw_soft) / span
        };
        r.clamp(0.0, self.hard_max)
    }

    pub fn in_soft_interior(&self, r: RiskCoord) -> bool {
        r <= self.soft_max
    }

    pub fn violates_hard(&self, r: RiskCoord) -> bool {
        r > self.hard_max
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct RiskVector {
    pub r_sat:    RiskCoord,
    pub r_do:     RiskCoord,
    pub r_flow:   RiskCoord,
    pub r_nutr:   RiskCoord,
    pub r_ecoli:  RiskCoord,
    pub r_pfas:   RiskCoord,
    // data-quality plane (rcalib) already used in your v2 core
    pub r_calib:  RiskCoord,
}

impl RiskVector {
    pub fn max_coord(&self) -> RiskCoord {
        self.r_sat
            .max(self.r_do)
            .max(self.r_flow)
            .max(self.r_nutr)
            .max(self.r_ecoli)
            .max(self.r_pfas)
            .max(self.r_calib)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LyapunovWeights {
    pub w_sat:    f32,
    pub w_do:     f32,
    pub w_flow:   f32,
    pub w_nutr:   f32,
    pub w_ecoli:  f32,
    pub w_pfas:   f32,
    pub w_calib:  f32,
}

impl LyapunovWeights {
    pub const fn phoenix_default() -> Self {
        Self {
            w_sat:   1.0,
            w_do:    1.3,
            w_flow:  0.8,
            w_nutr:  1.2,
            w_ecoli: 1.4,
            w_pfas:  1.5,
            w_calib: 0.8, // nonzero but below physical planes
        }
    }

    pub fn compute_residual(&self, r: &RiskVector) -> Residual {
        self.w_sat   * r.r_sat   * r.r_sat
      + self.w_do    * r.r_do    * r.r_do
      + self.w_flow  * r.r_flow  * r.r_flow
      + self.w_nutr  * r.r_nutr  * r.r_nutr
      + self.w_ecoli * r.r_ecoli * r.r_ecoli
      + self.w_pfas  * r.r_pfas  * r.r_pfas
      + self.w_calib * r.r_calib * r.r_calib
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SafeStepDecision {
    Accept,
    Derate,
    Stop,
}

#[derive(Clone, Copy, Debug)]
pub struct SafeStepConfig {
    pub epsilon: f32,
    pub max_risk_allowed: RiskCoord,  // e.g. 0.13 band you already use
}

impl SafeStepConfig {
    pub const fn phoenix_research_band() -> Self {
        Self {
            epsilon:          1.0e-4,
            max_risk_allowed: 0.13,
        }
    }
}

/// Discrete Lyapunov gate: when any corridor is violated, V_{t+1} must not exceed V_t.
pub fn eval_safestep(
    current_v: Residual,
    proposed_v: Residual,
    risk: &RiskVector,
    cfg: &SafeStepConfig,
) -> SafeStepDecision {
    let r_max = risk.max_coord();
    if r_max > cfg.max_risk_allowed {
        return SafeStepDecision::Stop;
    }
    if proposed_v <= current_v + cfg.epsilon {
        SafeStepDecision::Accept
    } else {
        SafeStepDecision::Derate
    }
}
