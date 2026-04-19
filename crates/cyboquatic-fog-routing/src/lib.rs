// File: crates/cyboquatic-fog-routing/src/lib.rs

#![forbid(unsafe_code)]

use std::time::Instant;
use cyboquatic_ecosafety_core::{RiskCoord, RiskVector, Residual, LyapunovWeights, residual};

#[derive(Clone, Copy, Debug)]
pub enum MediaClass {
    WaterOnly,
    WaterBiofilm,
    AirPlenum,
}

#[derive(Clone, Copy, Debug)]
pub struct CyboVariant {
    pub id: u64,
    pub energy_req_j: f64,
    pub safety_factor: f64,
    pub max_latency_ms: u64,
    pub media: MediaClass,
    /// Fraction of remaining hydraulic corridor this workload would consume.
    pub hydraulic_impact: f64,
    /// Nominal contribution to ΔV if routed to a neutral node.
    pub dv_t_nominal: f64,
    /// Expected carbon delta (positive = more emissions, negative = sequestration).
    pub d_carbon: f64,
    /// Expected biodiversity delta (negative = habitat improvement).
    pub d_biodiversity: f64,
}

#[derive(Clone, Copy, Debug)]
pub enum BioSurfaceMode {
    Raw,
    Preprocessed,
    Restricted,
}

#[derive(Clone, Copy, Debug)]
pub struct NodeShard {
    // Energy plane
    pub e_surplus_j: f64,
    pub p_margin_kw: f64,
    pub tailwind_mode: bool,
    pub dEdt_w: f64,
    // Hydraulics
    pub q_m3s: f64,
    pub hlr_m_per_h: f64,
    pub surcharge_risk_rx: f64,    // 0–1
    // Biology and media
    pub r_pathogen: f64,
    pub r_fouling: f64,
    pub r_cec: f64,
    pub bio_surface_mode: BioSurfaceMode,
    // Carbon + materials + biodiversity snapshots
    pub r_carbon: f64,
    pub r_materials: f64,
    pub r_biodiversity: f64,
    // Uncertainty
    pub r_sigma: f64,
    // Local residual view
    pub vt_local: f64,
    pub vt_trend: f64,
    // KER snapshots
    pub k_score: f64,
    pub e_score: f64,
    pub r_score: f64,
}

#[derive(Clone, Copy, Debug)]
pub enum RouteDecision {
    Accept,
    Reject,
    Reroute,
}

#[derive(Clone, Copy, Debug)]
pub struct RoutingContext {
    pub vt_global: f64,
    pub vt_global_next_max: f64,
    pub now: Instant,
    pub weights: LyapunovWeights,
}

/// Energy predicate: require tailwind and positive post‑allocation surplus.
fn tailwind_valid(node: &NodeShard, variant: &CyboVariant) -> bool {
    if !node.tailwind_mode {
        return false;
    }
    let sf = variant.safety_factor.max(1.0);
    let required = variant.energy_req_j * sf;
    node.e_surplus_j - required > 0.0 && node.p_margin_kw > 0.0 && node.dEdt_w >= 0.0
}

/// Biology predicate with gold corridors stricter than legal.
fn biosurface_ok(node: &NodeShard, variant: &CyboVariant) -> bool {
    use MediaClass::*;
    use BioSurfaceMode::*;

    if let Restricted = node.bio_surface_mode {
        // Only air‑plenum tasks on restricted surfaces.
        return matches!(variant.media, AirPlenum);
    }

    let r_thresh = 0.5_f64; // illustrative gold band

    match variant.media {
        AirPlenum => node.r_pathogen <= r_thresh,
        WaterOnly | WaterBiofilm => {
            matches!(node.bio_surface_mode, Preprocessed)
                && node.r_pathogen <= r_thresh
                && node.r_fouling <= r_thresh
                && node.r_cec <= r_thresh
        }
    }
}

/// Hydraulics predicate: keep surcharge below corridor closure.
fn hydraulics_ok(node: &NodeShard, variant: &CyboVariant) -> bool {
    let impact = variant.hydraulic_impact.max(0.0);
    let rx = node.surcharge_risk_rx.max(0.0);
    let predicted = rx + impact;
    predicted < 1.0
}

/// Lyapunov predicate: V_{t+1} ≤ V_t and under configured bound.
fn lyapunov_ok(node: &NodeShard, variant: &CyboVariant, ctx: &RoutingContext) -> bool {
    let rv_prev = RiskVector {
        r_energy: RiskCoord::clamped(0.0), // per‑plane derivation below
        r_hydraulics: RiskCoord::clamped(node.surcharge_risk_rx),
        r_biology: RiskCoord::clamped(node.r_pathogen.max(node.r_fouling).max(node.r_cec)),
        r_carbon: RiskCoord::clamped(node.r_carbon),
        r_materials: RiskCoord::clamped(node.r_materials),
        r_biodiversity: RiskCoord::clamped(node.r_biodiversity),
        r_sigma: RiskCoord::clamped(node.r_sigma),
    };
    let vt_prev = Residual { value: ctx.vt_global };

    // Approximate impact of variant on three key planes.
    let rv_next = RiskVector {
        r_energy: RiskCoord::clamped(0.0), // energy tailwind is separately checked
        r_hydraulics: RiskCoord::clamped(node.surcharge_risk_rx + variant.hydraulic_impact),
        r_biology: rv_prev.r_biology,
        r_carbon: RiskCoord::clamped((node.r_carbon + variant.d_carbon).max(0.0)),
        r_materials: rv_prev.r_materials,
        r_biodiversity: RiskCoord::clamped((node.r_biodiversity + variant.d_biodiversity).max(0.0)),
        r_sigma: rv_prev.r_sigma,
    };
    let vt_est = residual(&rv_next, &ctx.weights);

    let vtnext_est = vt_est.value;
    let vt_bound_ok = vtnext_est <= ctx.vt_global_next_max;
    let non_increasing = vtnext_est <= ctx.vt_global;
    let local_trend_ok = node.vt_trend <= 0.0;

    vt_bound_ok && non_increasing && local_trend_ok
}

/// Composite routing rule.
pub fn route_variant(
    variant: &CyboVariant,
    node: &NodeShard,
    ctx: &RoutingContext,
) -> RouteDecision {
    if !tailwind_valid(node, variant) {
        return RouteDecision::Reroute;
    }
    if !biosurface_ok(node, variant) {
        return RouteDecision::Reroute;
    }
    if !hydraulics_ok(node, variant) {
        return RouteDecision::Reroute;
    }
    if !lyapunov_ok(node, variant, ctx) {
        return RouteDecision::Reject;
    }
    RouteDecision::Accept
}
