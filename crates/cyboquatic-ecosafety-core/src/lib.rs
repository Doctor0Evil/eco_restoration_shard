//! cyboquatic-ecosafety-core
//! Rust ecosafety spine for Cyboquatic industrial machinery (rx / Vt / KER).
//! All domains (MAR, FlowVac, FOG, wetlands, trays, air plenums) parameterize this grammar.[file:10]

#![forbid(unsafe_code)]

use std::marker::PhantomData;
use std::time::{Duration, Instant};

/// Dimensionless risk scalar r ∈ [0, 1].
pub type RiskScalar = f64;

/// Normalized risk coordinate r ∈ [0, 1], clamped.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RiskCoord(RiskScalar);

impl RiskCoord {
    /// Create a clamped coordinate from a raw scalar.
    pub fn new(raw: RiskScalar) -> Self {
        RiskCoord(raw.clamp(0.0, 1.0))
    }

    /// Get the underlying scalar value.
    pub fn value(self) -> RiskScalar {
        self.0
    }

    /// Maximum of two coordinates.
    pub fn max(self, other: RiskCoord) -> RiskCoord {
        RiskCoord::new(self.0.max(other.0))
    }
}

/// Corridor bands for a single physical metric in raw units
/// (e.g., mg/L, m/d, days). Safe ≤ Gold ≤ Hard.[file:7]
#[derive(Clone, Copy, Debug)]
pub struct CorridorBands {
    pub safe: RiskScalar,
    pub gold: RiskScalar,
    pub hard: RiskScalar,
}

impl CorridorBands {
    pub fn new(safe: RiskScalar, gold: RiskScalar, hard: RiskScalar) -> Self {
        let bands = Self { safe, gold, hard };
        bands.assert_well_formed();
        bands
    }

    pub fn assert_well_formed(&self) {
        assert!(
            self.safe <= self.gold && self.gold <= self.hard,
            "CorridorBands must satisfy safe ≤ gold ≤ hard"
        );
    }

    /// Piecewise-linear normalization into RiskCoord using safe/gold/hard bands.[file:7]
    ///
    /// safe → 0.0
    /// gold → 0.5
    /// hard → 1.0
    pub fn normalize(&self, x: RiskScalar) -> RiskCoord {
        self.assert_well_formed();

        if x <= self.safe {
            return RiskCoord::new(0.0);
        }
        if x >= self.hard {
            return RiskCoord::new(1.0);
        }

        if x <= self.gold {
            // Gentle slope between safe and gold.
            let num = x - self.safe;
            let den = (self.gold - self.safe).max(RiskScalar::EPSILON);
            return RiskCoord::new(0.5 * num / den);
        }

        // Steeper slope from gold to hard, mapping into (0.5, 1.0).
        let num = x - self.gold;
        let den = (self.hard - self.gold).max(RiskScalar::EPSILON);
        RiskCoord::new(0.5 + 0.5 * num / den)
    }

    /// Simple check: any value at or below hard band is considered within corridor.
    pub fn is_within(&self, r: RiskCoord) -> bool {
        r.value() <= self.hard
    }

    /// Check if value sits inside the "gold" interior band.
    pub fn is_gold(&self, r: RiskCoord) -> bool {
        r.value() <= self.gold
    }
}

/// Canonical planes for Cyboquatic machinery:
/// energy, hydraulics, biology, carbon, materials, biodiversity, and uncertainty σ.[file:10][file:7]
///
/// All coordinates are normalized to [0, 1] where 0 = best, 1 = worst.
#[derive(Clone, Copy, Debug)]
pub struct RiskVector {
    pub energy: RiskCoord,
    pub hydraulics: RiskCoord,
    pub biology: RiskCoord,
    pub carbon: RiskCoord,
    pub materials: RiskCoord,
    pub biodiversity: RiskCoord,
    /// Sensor / model uncertainty (rsigma, rcalib).
    pub sigma: RiskCoord,
}

impl RiskVector {
    /// Max over all coordinates, including uncertainty.
    pub fn max_coord(&self) -> RiskCoord {
        let vals = [
            self.energy.value(),
            self.hydraulics.value(),
            self.biology.value(),
            self.carbon.value(),
            self.materials.value(),
            self.biodiversity.value(),
            self.sigma.value(),
        ];
        RiskCoord::new(vals.into_iter().fold(0.0, RiskScalar::max))
    }

    /// Max over physical planes only (excludes uncertainty).
    pub fn max_physical(&self) -> RiskCoord {
        let vals = [
            self.energy.value(),
            self.hydraulics.value(),
            self.biology.value(),
            self.carbon.value(),
            self.materials.value(),
            self.biodiversity.value(),
        ];
        RiskCoord::new(vals.into_iter().fold(0.0, RiskScalar::max))
    }
}

/// Quadratic Lyapunov residual V_t = Σ w_j r_j^2 over all planes.[file:10][file:7]
#[derive(Clone, Copy, Debug)]
pub struct Residual {
    pub value: RiskScalar,
}

#[derive(Clone, Copy, Debug)]
pub struct LyapunovWeights {
    pub w_energy: RiskScalar,
    pub w_hydraulics: RiskScalar,
    pub w_biology: RiskScalar,
    pub w_carbon: RiskScalar,
    pub w_materials: RiskScalar,
    pub w_biodiversity: RiskScalar,
    pub w_sigma: RiskScalar,
}

impl LyapunovWeights {
    pub fn assert_non_negative(&self) {
        assert!(self.w_energy >= 0.0);
        assert!(self.w_hydraulics >= 0.0);
        assert!(self.w_biology >= 0.0);
        assert!(self.w_carbon >= 0.0);
        assert!(self.w_materials >= 0.0);
        assert!(self.w_biodiversity >= 0.0);
        assert!(self.w_sigma >= 0.0);
    }

    /// Default ordering that treats carbon drift and habitat loss as severe,
    /// and gives uncertainty a nonzero weight.[file:10][file:11]
    pub fn default_carbon_negative() -> Self {
        Self {
            w_energy: 1.0,
            w_hydraulics: 1.0,
            w_biology: 1.2,
            w_carbon: 1.3,
            w_materials: 1.1,
            w_biodiversity: 1.1,
            w_sigma: 0.8,
        }
    }
}

/// Compute Lyapunov residual V_t = Σ w_j r_j^2.
pub fn residual(rv: &RiskVector, w: &LyapunovWeights) -> Residual {
    w.assert_non_negative();
    let sq = |x: RiskScalar| x * x;

    let v = w.w_energy * sq(rv.energy.value())
        + w.w_hydraulics * sq(rv.hydraulics.value())
        + w.w_biology * sq(rv.biology.value())
        + w.w_carbon * sq(rv.carbon.value())
        + w.w_materials * sq(rv.materials.value())
        + w.w_biodiversity * sq(rv.biodiversity.value())
        + w.w_sigma * sq(rv.sigma.value());

    Residual { value: v }
}

/// Per‑plane corridor bands: safe < gold < hard in normalized units.
#[derive(Clone, Copy, Debug)]
pub struct NormalizedBands {
    pub safe: RiskScalar,
    pub gold: RiskScalar,
    pub hard: RiskScalar,
}

impl NormalizedBands {
    pub fn new(safe: RiskScalar, gold: RiskScalar, hard: RiskScalar) -> Self {
        assert!(
            safe <= gold && gold <= hard,
            "NormalizedBands must satisfy safe ≤ gold ≤ hard"
        );
        Self { safe, gold, hard }
    }

    /// Any r ≤ hard is in corridor.
    pub fn is_within(&self, r: RiskCoord) -> bool {
        r.value() <= self.hard
    }

    /// Any r ≤ gold is considered "gold corridor".
    pub fn is_gold(&self, r: RiskCoord) -> bool {
        r.value() <= self.gold
    }
}

/// Minimal view of prior and candidate step for safestep.
#[derive(Clone, Copy, Debug)]
pub struct StepContext {
    pub vt_prev: Residual,
    pub vt_next: Residual,
}

/// Decision from safestep.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SafeStepDecision {
    /// Step accepted, actuation allowed.
    Accept,
    /// Step rejected but not catastrophic; derate or hold.
    Reject,
}

/// Hard governance thresholds for promotion (KER).
#[derive(Clone, Copy, Debug)]
pub struct KerThresholds {
    pub k_min: RiskScalar,
    pub e_min: RiskScalar,
    pub r_max: RiskScalar,
}

impl KerThresholds {
    /// Production thresholds: K ≥ 0.90, E ≥ 0.90, R ≤ 0.13.[file:10][file:7]
    pub fn prod_defaults() -> Self {
        Self {
            k_min: 0.90,
            e_min: 0.90,
            r_max: 0.13,
        }
    }
}

/// Rolling KER metrics over a window.[file:10]
#[derive(Clone, Copy, Debug)]
pub struct KerWindow {
    pub k: RiskScalar, // Knowledge-factor
    pub e: RiskScalar, // Eco-impact
    pub r: RiskScalar, // Risk-of-harm (max risk coordinate)
}

impl KerWindow {
    /// A simple mapping: E = 1 − R with clamp.
    /// k is taken as the fraction of Lyapunov-safe steps in the window.[file:10]
    pub fn from_risk(max_coord: RiskScalar, fraction_safe_steps: RiskScalar) -> Self {
        let r = max_coord.clamp(0.0, 1.0);
        let k = fraction_safe_steps.clamp(0.0, 1.0);
        let e = (1.0 - r).clamp(0.0, 1.0);
        Self { k, e, r }
    }

    pub fn meets(&self, thr: &KerThresholds) -> bool {
        self.k >= thr.k_min && self.e >= thr.e_min && self.r <= thr.r_max
    }
}

/// Type‑level guard: no actuation without risk.
/// Enforces V_{t+1} ≤ V_t + epsilon.
pub fn safestep(ctx: StepContext, epsilon: RiskScalar) -> SafeStepDecision {
    if ctx.vt_next.value <= ctx.vt_prev.value + epsilon {
        SafeStepDecision::Accept
    } else {
        SafeStepDecision::Reject
    }
}

/// For controllers: every proposal must attach a RiskVector.
pub trait SafeController {
    type Actuation;

    /// Propose an actuation over a time step `dt`, returning both
    /// the proposed actuation and a full RiskVector for ecosafety evaluation.[file:7]
    fn propose_step(&mut self, dt: Duration) -> (Self::Actuation, RiskVector);

    /// Default ecosafety gate: recompute V_t, run safestep, return decision and new residual.
    ///
    /// Implementors must ensure that physical actuators are only called when
    /// the decision is `SafeStepDecision::Accept`.
    fn apply_if_safe(
        &mut self,
        act: Self::Actuation,
        rv: RiskVector,
        w: &LyapunovWeights,
        vt_prev: Residual,
        eps: RiskScalar,
    ) -> (SafeStepDecision, Residual) {
        let vt_next = residual(&rv, w);
        let decision = safestep(StepContext { vt_prev, vt_next }, eps);
        (decision, vt_next)
    }
}

/// Ecosafety corridor decision used by higher-level kernels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CorridorDecision {
    /// Inside all corridors, Lyapunov-safe, actuation allowed.
    Normal,
    /// Soft breach: derate, adjust, or hold; no actuation this step.
    Derate,
    /// Hard breach: stop, require governance intervention.
    Stop,
}

/// Rolling-window KER metrics with explicit temporal logic.
/// This struct is used by controllers / planners to evaluate longer trajectories.[file:7][file:10]
#[derive(Clone, Copy, Debug)]
pub struct KerTriad {
    pub k_knowledge: RiskScalar,
    pub e_eco_impact: RiskScalar,
    pub r_risk_of_harm: RiskScalar,
}

#[derive(Clone, Debug)]
pub struct KerWindowTracker {
    window_start: Instant,
    window_duration: Duration,
    steps_total: u64,
    steps_lyap_safe: u64,
    r_max: RiskScalar,
}

impl KerWindowTracker {
    pub fn new(window_duration: Duration) -> Self {
        KerWindowTracker {
            window_start: Instant::now(),
            window_duration,
            steps_total: 0,
            steps_lyap_safe: 0,
            r_max: 0.0,
        }
    }

    /// Observe one step, updating K/E/R statistics.
    pub fn observe_step(&mut self, risk_vec: &RiskVector, lyap_safe: bool) {
        self.steps_total = self.steps_total.saturating_add(1);
        if lyap_safe {
            self.steps_lyap_safe = self.steps_lyap_safe.saturating_add(1);
        }
        let r = risk_vec.max_coord().value();
        if r > self.r_max {
            self.r_max = r;
        }

        if self.window_start.elapsed() >= self.window_duration {
            // Reset window but carry last risk as baseline.
            self.window_start = Instant::now();
            self.steps_total = 0;
            self.steps_lyap_safe = 0;
            self.r_max = r;
        }
    }

    pub fn triad(&self) -> KerTriad {
        let k = if self.steps_total == 0 {
            1.0
        } else {
            self.steps_lyap_safe as RiskScalar / self.steps_total as RiskScalar
        };
        let r = self.r_max.clamp(0.0, 1.0);
        let e = (1.0 - r).clamp(0.0, 1.0);
        KerTriad {
            k_knowledge: k,
            e_eco_impact: e,
            r_risk_of_harm: r,
        }
    }

    /// Production gate: K ≥ 0.90, E ≥ 0.90, R ≤ 0.13.[file:10][file:7]
    pub fn production_admissible(&self) -> bool {
        let triad = self.triad();
        triad.k_knowledge >= 0.90
            && triad.e_eco_impact >= 0.90
            && triad.r_risk_of_harm <= 0.13
    }
}

/// Trait every Cyboquatic controller must implement:
/// no action without a RiskVector, and every step is Lyapunov-gated.[file:7]
pub trait EcosafetyController<S, A> {
    /// Propose an actuation given current plant state.
    /// Must also emit a full RiskVector for ecosafety evaluation.
    fn propose_step(&mut self, state: &S) -> (A, RiskVector);
}

/// Ecosafety kernel that wraps controllers and enforces corridors and Lyapunov invariant.[file:7][file:10]
pub struct EcoSafetyKernel<S, A> {
    pub residual_prev: Residual,
    pub weights: LyapunovWeights,
    pub eps_vt: RiskScalar,
    pub window: KerWindowTracker,
    _phantom_s: PhantomData<S>,
    _phantom_a: PhantomData<A>,
}

impl<S, A> EcoSafetyKernel<S, A> {
    pub fn new(weights: LyapunovWeights, eps_vt: RiskScalar, window_duration: Duration) -> Self {
        Self {
            residual_prev: Residual { value: 0.0 },
            weights,
            eps_vt,
            window: KerWindowTracker::new(window_duration),
            _phantom_s: PhantomData,
            _phantom_a: PhantomData,
        }
    }

    /// Evaluate a proposed step and return (CorridorDecision, maybe_actuation).
    /// Actuation is None if the step is rejected or derated.[file:7]
    pub fn evaluate_step<C>(
        &mut self,
        controller: &mut C,
        state: &S,
    ) -> (CorridorDecision, Option<A>)
    where
        C: EcosafetyController<S, A>,
    {
        let (act, rv) = controller.propose_step(state);
        let residual_new = residual(&rv, &self.weights);

        let r_max = rv.max_coord().value();
        // Hard violation if any plane, including uncertainty, reaches or exceeds 1.0.
        let hard_violation = (r_max - 1.0).abs() < 1e-9 || r_max > 1.0;

        let lyap_ok = residual_new.value <= self.residual_prev.value + self.eps_vt;

        // Only count as Lyapunov-safe if we are not violating hard corridors.
        self.window
            .observe_step(&rv, lyap_ok && !hard_violation);

        // Update residual for next step.
        self.residual_prev = residual_new;

        if hard_violation {
            return (CorridorDecision::Stop, None);
        }

        if !lyap_ok {
            // Soft breach: derate and do not actuate.
            return (CorridorDecision::Derate, None);
        }

        // Safe step: actuation allowed.
        (CorridorDecision::Normal, Some(act))
    }
}
