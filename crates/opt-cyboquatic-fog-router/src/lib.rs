//! opt-cyboquatic-fog-router
//! FOG-style routing for Cyboquatic workloads with energy, hydraulic, bio, and Lyapunov gates.

#![forbid(unsafe_code)]

use std::time::Instant;

use cyboquatic_ecosafety_core::{RiskCoord, RiskVector, ResidualState, ResidualWeights};

#[derive(Clone, Copy, Debug)]
pub enum MediaClass {
    WaterOnly,
    WaterBiofilm,
    AirPlenum,
}

#[derive(Clone, Copy, Debug)]
pub struct CyboWorkload {
    pub id: u64,
    pub energy_req_j: f64,
    pub safety_factor: f64,
    pub hydraulic_impact: f64, // normalized fraction of remaining corridor margin.[file:21]
    pub dv_t_nominal: f64,     // expected Lyapunov delta for neutral node.
    pub media: MediaClass,
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
    pub dE_dt_w: f64,
    // Hydraulics
    pub q_m3s: f64,
    pub surcharge_risk_rx: f64,
    // Biology
    pub r_pathogen: f64,
    pub r_fouling: f64,
    pub r_cec: f64,
    pub biosurface_mode: BioSurfaceMode,
    // Residual and KER views
    pub vt_local: f64,
    pub vt_trend: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
}

/// Energy predicate: ensure true surplus after workload energy + safety factor.[file:21]
pub fn tailwind_valid(node: &NodeShard, workload: &CyboWorkload) -> bool {
    if !node.tailwind_mode {
        return false;
    }
    let required = workload.energy_req_j * workload.safety_factor.max(1.0);
    node.e_surplus_j > required && node.p_margin_kw > 0.0 && node.dE_dt_w >= 0.0
}

/// Biological substrate predicate: gold corridors for bio-contact.[file:21]
pub fn biosurface_ok(node: &NodeShard, workload: &CyboWorkload) -> bool {
    match workload.media {
        MediaClass::AirPlenum => {
            // Allow only if pathogen risk is below corridor.
            node.r_pathogen <= 0.5
        }
        MediaClass::WaterOnly | MediaClass::WaterBiofilm => {
            if !matches!(node.biosurface_mode, BioSurfaceMode::Preprocessed) {
                return false;
            }
            let r_thresh = 0.5;
            node.r_pathogen <= r_thresh &&
                node.r_fouling <= r_thresh &&
                node.r_cec <= r_thresh
        }
    }
}

/// Hydraulic predicate: keep surcharge risk within corridor.[file:21]
pub fn hydraulic_ok(node: &NodeShard, workload: &CyboWorkload) -> bool {
    let impact = workload.hydraulic_impact.max(0.0);
    let rx = node.surcharge_risk_rx.max(0.0);
    let predicted = rx + impact;
    predicted < 1.0
}

/// Lyapunov predicate: enforce V_{t+1} ≤ V_t and global bound.[file:21][file:3]
pub fn lyapunov_ok(
    node: &NodeShard,
    workload: &CyboWorkload,
    ctx: &RoutingContext,
    weights: &ResidualWeights,
) -> bool {
    let rv = RiskVector {
        energy: RiskCoord::new(0.0),    // workload-specific refinement possible
        hydraulics: RiskCoord::new(node.surcharge_risk_rx),
        biology: RiskCoord::new(node.r_pathogen.max(node.r_fouling).max(node.r_cec)),
        carbon: RiskCoord::new(0.0),
        materials: RiskCoord::new(0.0),
    };
    let residual_prev = ResidualState { vt: ctx.vt_global };
    let mut residual_new = ResidualState::from_risks(&rv, weights);
    residual_new.vt += workload.dv_t_nominal;

    let local_ok = residual_new.safestep_ok(&residual_prev, 0.0);
    let global_ok = residual_new.vt <= ctx.vt_global_next_max && node.vt_trend <= 0.0;
    local_ok && global_ok
}

/// Composite routing rule over all four planes.[file:21]
pub fn route_workload(
    workload: &CyboWorkload,
    node: &NodeShard,
    ctx: &RoutingContext,
    weights: &ResidualWeights,
) -> RouteDecision {
    if !tailwind_valid(node, workload) {
        return RouteDecision::Reroute;
    }
    if !biosurface_ok(node, workload) {
        return RouteDecision::Reroute;
    }
    if !hydraulic_ok(node, workload) {
        return RouteDecision::Reroute;
    }
    if !lyapunov_ok(node, workload, ctx, weights) {
        return RouteDecision::Reject;
    }
    RouteDecision::Accept
}
