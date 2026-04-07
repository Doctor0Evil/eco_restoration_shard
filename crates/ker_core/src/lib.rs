//! Core ecosafety math kernel for Artemis, Cyboquatic, and Quantum VMs.
//! Plane-agnostic K/E/R, Lyapunov, and trust-weighted aggregation.
//!
//! Constraints:
//! - no unsafe code
//! - deny warnings and clippy lints
#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(clippy::all)]

use std::fmt;

/// Normalized risk coordinate r_j ∈ [0,1].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RiskCoord {
    pub name: &'static str,
    pub value: f64,
}

impl RiskCoord {
    pub fn new(name: &'static str, raw: f64) -> Self {
        Self {
            name,
            value: raw.clamp(0.0, 1.0),
        }
    }
}

/// Corridor bands for a single normalized coordinate.
/// 0.0 ≤ safe ≤ gold ≤ hard ≤ 1.0
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CorridorBands {
    pub safe: f64,
    pub gold: f64,
    pub hard: f64,
}

impl CorridorBands {
    pub fn new(safe: f64, gold: f64, hard: f64) -> Self {
        assert!((0.0..=1.0).contains(&safe));
        assert!((safe..=1.0).contains(&gold));
        assert!((gold..=1.0).contains(&hard));
        Self { safe, gold, hard }
    }

    pub fn band(&self, r: f64) -> CorridorBand {
        let r = r.clamp(0.0, 1.0);
        if r <= self.safe {
            CorridorBand::Safe
        } else if r <= self.gold {
            CorridorBand::Gold
        } else if r <= self.hard {
            CorridorBand::NearHard
        } else {
            CorridorBand::HardBreach
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CorridorBand {
    Safe,
    Gold,
    NearHard,
    HardBreach,
}

/// Lyapunov weights for a set of named risk coordinates.
#[derive(Clone, Debug)]
pub struct LyapunovWeights {
    pub names: Vec<&'static str>,
    pub weights: Vec<f64>,
}

impl LyapunovWeights {
    pub fn new(names: Vec<&'static str>, weights: Vec<f64>) -> Self {
        assert_eq!(names.len(), weights.len());
        for w in &weights {
            assert!(*w >= 0.0);
        }
        Self { names, weights }
    }

    pub fn weight_for(&self, name: &str) -> f64 {
        self.names
            .iter()
            .zip(self.weights.iter())
            .find(|(n, _)| **n == name)
            .map(|(_, w)| *w)
            .unwrap_or(0.0)
    }
}

/// Plane-agnostic metric bundle used across kernels.
#[derive(Clone, Debug)]
pub struct MetricFields {
    pub rx: Vec<RiskCoord>,
    pub vt: f64,
    pub vt_next: f64,
    pub k: f64,
    pub e: f64,
    pub r: f64,
}

impl MetricFields {
    pub fn new(rx: Vec<RiskCoord>, vt: f64, vt_next: f64, k: f64, e: f64, r: f64) -> Self {
        Self {
            rx,
            vt,
            vt_next,
            k,
            e,
            r,
        }
    }
}

/// Discrete Lyapunov residual V_t = Σ_j w_j r_j^2.
pub fn lyapunov_residual(rx: &[RiskCoord], weights: &LyapunovWeights) -> f64 {
    rx.iter()
        .map(|coord| {
            let w = weights.weight_for(coord.name);
            w * coord.value * coord.value
        })
        .sum()
}

/// Step classification under Lyapunov and corridor invariants.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResidualStep {
    Accept,
    Derate,
    Stop,
}

/// Enforce V_{t+1} ≤ V_t outside a small interior band ε.
pub fn safestep_residual(vt: f64, vt_next: f64, epsilon: f64) -> ResidualStep {
    if vt_next > vt + epsilon {
        ResidualStep::Stop
    } else if vt_next > vt {
        ResidualStep::Derate
    } else {
        ResidualStep::Accept
    }
}

/// Check for any hard corridor breach.
pub fn check_corridors(
    rx: &[RiskCoord],
    bands: &[(&'static str, CorridorBands)],
) -> ResidualStep {
    for coord in rx {
        if let Some((_, b)) = bands.iter().find(|(n, _)| *n == coord.name) {
            if b.band(coord.value) == CorridorBand::HardBreach {
                return ResidualStep::Stop;
            }
        }
    }
    ResidualStep::Accept
}

/// Combined safestep used by controllers in all planes.
pub fn safestep_combined(
    vt: f64,
    vt_next: f64,
    epsilon: f64,
    rx: &[RiskCoord],
    bands: &[(&'static str, CorridorBands)],
) -> ResidualStep {
    let corridor_step = check_corridors(rx, bands);
    if matches!(corridor_step, ResidualStep::Stop) {
        return ResidualStep::Stop;
    }
    safestep_residual(vt, vt_next, epsilon)
}

/// Monotone K/E/R update: K,E non-decreasing, R non-increasing.
pub fn update_ker(
    k_old: f64,
    e_old: f64,
    r_old: f64,
    k_new: f64,
    e_new: f64,
    r_new: f64,
) -> (f64, f64, f64) {
    let k = k_new.max(k_old);
    let e = e_new.max(e_old);
    let r = r_new.min(r_old);
    (k, e, r)
}

/// Trust weight w_t = clamp(D_t (1 - r_uncertainty) r_sensor).
#[derive(Clone, Copy, Debug)]
pub struct TrustInputs {
    pub r_sensor: f64,
    pub r_uncertainty: f64,
    pub d_t: f64,
}

pub fn trust_weight_step(inputs: TrustInputs) -> f64 {
    let r_sensor = inputs.r_sensor.clamp(0.0, 1.0);
    let r_unc = inputs.r_uncertainty.clamp(0.0, 1.0);
    let d = inputs.d_t.clamp(0.0, 1.0);
    let raw = d * (1.0 - r_unc) * r_sensor;
    raw.clamp(0.0, 1.0)
}

/// Trust-adjusted K_tight over a window W.
pub fn k_trust(
    rsafe: &[bool],
    w: &[f64],
    eps: f64,
) -> f64 {
    assert_eq!(rsafe.len(), w.len());
    let mut num = 0.0;
    let mut den = 0.0;
    for (ok, wt) in rsafe.iter().zip(w.iter()) {
        let wt = wt.max(0.0);
        den += wt;
        if *ok {
            num += wt;
        }
    }
    num / (den + eps)
}

/// Trust-adjusted E_cap over a window W.
pub fn e_trust(
    e_t: &[f64],
    r_t: &[f64],
    w: &[f64],
    eps: f64,
) -> f64 {
    assert_eq!(e_t.len(), r_t.len());
    assert_eq!(e_t.len(), w.len());
    let mut num = 0.0;
    let mut den = 0.0;
    for ((e, r), wt) in e_t.iter().zip(r_t.iter()).zip(w.iter()) {
        let wt = wt.max(0.0);
        let one_minus_r = (1.0 - *r).clamp(0.0, 1.0);
        num += e * one_minus_r * wt;
        den += one_minus_r * wt;
    }
    num / (den + eps)
}

impl fmt::Display for MetricFields {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "MetricFields")?;
        writeln!(f, "  vt     = {:.6}", self.vt)?;
        writeln!(f, "  vt_next= {:.6}", self.vt_next)?;
        writeln!(f, "  K/E/R  = {:.4} / {:.4} / {:.4}", self.k, self.e, self.r)?;
        writeln!(f, "  rx:")?;
        for coord in &self.rx {
            writeln!(f, "    {} = {:.6}", coord.name, coord.value)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn residual_nonnegative() {
        let rx = vec![RiskCoord::new("r_energy", 0.3), RiskCoord::new("r_rf", 0.2)];
        let weights = LyapunovWeights::new(vec!["r_energy", "r_rf"], vec![1.0, 0.5]);
        let vt = lyapunov_residual(&rx, &weights);
        assert!(vt >= 0.0);
    }

    #[test]
    fn corridor_bands_ordering() {
        let bands = CorridorBands::new(0.3, 0.6, 0.9);
        assert_eq!(bands.band(0.1), CorridorBand::Safe);
        assert_eq!(bands.band(0.4), CorridorBand::Gold);
        assert_eq!(bands.band(0.7), CorridorBand::NearHard);
        assert_eq!(bands.band(0.95), CorridorBand::HardBreach);
    }

    #[test]
    fn safestep_combined_behavior() {
        let rx = vec![RiskCoord::new("r_energy", 0.8)];
        let bands = vec![("r_energy", CorridorBands::new(0.3, 0.6, 0.9))];
        let step_ok = safestep_combined(0.5, 0.49, 1e-6, &rx, &bands);
        assert_eq!(step_ok, ResidualStep::Accept);
        let step_stop = safestep_combined(0.5, 0.6, 1e-6, &rx, &bands);
        assert_eq!(step_stop, ResidualStep::Stop);
    }

    #[test]
    fn ker_update_monotone() {
        let (k, e, r) = update_ker(0.90, 0.88, 0.16, 0.92, 0.89, 0.14);
        assert!((k - 0.92).abs() < 1e-9);
        assert!((e - 0.89).abs() < 1e-9);
        assert!((r - 0.14).abs() < 1e-9);
    }

    #[test]
    fn trust_weight_basic() {
        let w = trust_weight_step(TrustInputs {
            r_sensor: 0.9,
            r_uncertainty: 0.1,
            d_t: 0.95,
        });
        assert!(w > 0.0 && w <= 1.0);
    }

    #[test]
    fn k_e_trust_windows() {
        let rsafe = vec![true, false, true];
        let w = vec![0.9, 0.2, 0.5];
        let k = k_trust(&rsafe, &w, 1e-9);
        assert!(k >= 0.0 && k <= 1.0);

        let e_t = vec![0.9, 0.8, 0.7];
        let r_t = vec![0.1, 0.2, 0.3];
        let e = e_trust(&e_t, &r_t, &w, 1e-9);
        assert!(e >= 0.0 && e <= 1.0);
    }
}
