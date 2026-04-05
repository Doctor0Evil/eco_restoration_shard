// File: services/cyboquatic-fog-router/src/lib.rs

use std::time::Instant;
use cyboquatic_ecosafety_core::{Residual, NodeAction};

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
    /// Fraction of remaining hydraulic corridor consumed (0–1).
    pub hydraulic_impact: f64,
    /// Expected ΔV_t if routed to neutral node.
    pub dv_t_nominal: f64,
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
    pub surcharge_risk_rx: f64, // 0–1
    // Biology
    pub r_pathogen: f64,
    pub r_fouling: f64,
    pub r_cec: f64,
    pub bio_surface_mode: BioSurfaceMode,
    // Local residual view
    pub vt_local: f64,
    pub vt_trend: f64,
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
}

fn clamp01(x: f64) -> f64 {
    if x < 0.0 { 0.0 } else if x > 1.0 { 1.0 } else { x }
}

fn tailwind_valid(node: &NodeShard, variant: &CyboVariant) -> bool {
    if !node.tailwind_mode {
        return false;
    }
    let required = variant.energy_req_j * variant.safety_factor.max(1.0);
    node.e_surplus_j > required && node.p_margin_kw > 0.0 && node.dE_dt_w <= 0.0
}

fn biosurface_ok(node: &NodeShard, variant: &CyboVariant) -> bool {
    match node.bio_surface_mode {
        BioSurfaceMode::Restricted => matches!(variant.media, MediaClass::AirPlenum),
        BioSurfaceMode::Raw | BioSurfaceMode::Preprocessed => {
            let r_thresh = 0.5;
            match variant.media {
                MediaClass::AirPlenum => node.r_pathogen <= r_thresh,
                MediaClass::WaterOnly | MediaClass::WaterBiofilm => {
                    matches!(node.bio_surface_mode, BioSurfaceMode::Preprocessed) &&
                    node.r_pathogen <= r_thresh &&
                    node.r_fouling <= r_thresh &&
                    node.r_cec <= r_thresh
                }
            }
        }
    }
}

fn hydraulic_ok(node: &NodeShard, variant: &CyboVariant) -> bool {
    let impact = variant.hydraulic_impact.max(0.0);
    let rx = clamp01(node.surcharge_risk_rx);
    let predicted = rx + impact;
    predicted <= 1.0
}

fn lyapunov_ok(node: &NodeShard, variant: &CyboVariant, ctx: &RoutingContext) -> bool {
    let dv_local = variant.dv_t_nominal;
    let vt_next_est = ctx.vt_global + dv_local;
    vt_next_est <= ctx.vt_global_next_max && dv_local + node.vt_trend <= 0.0
}

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
    if !hydraulic_ok(node, variant) {
        return RouteDecision::Reroute;
    }
    if !lyapunov_ok(node, variant, ctx) {
        return RouteDecision::Reject;
    }
    RouteDecision::Accept
}

// Example: mapping FOG decision into ecosafety NodeAction envelope.
pub fn as_node_action(decision: RouteDecision) -> NodeAction {
    match decision {
        RouteDecision::Accept => NodeAction::Normal,
        RouteDecision::Reroute => NodeAction::Derate,
        RouteDecision::Reject => NodeAction::Stop,
    }
}
