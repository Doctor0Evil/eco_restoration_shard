// EcoNetTerminalAgent/src/resource_corridors.rs
// Core ecosafety grammar for local CPU/RAM/NET corridors.

#![forbid(unsafe_code)]

#[derive(Clone, Debug)]
pub struct CorridorBands {
    pub var_id: &'static str,
    pub safe: f64,
    pub gold: f64,
    pub hard: f64,
    pub weight: f64,
    pub lyap_channel: u8,
}

#[derive(Clone, Debug)]
pub struct RiskCoord {
    pub value: f64,          // rx in [0,1]
    pub bands: CorridorBands,
}

#[derive(Clone, Debug)]
pub struct Residual {
    pub vt: f64,
    pub coords: Vec<RiskCoord>,
}

impl Residual {
    pub fn recompute(&mut self) {
        self.vt = self.coords
            .iter()
            .map(|c| c.value * c.bands.weight)
            .sum::<f64>();
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CorridorDecision {
    Ok,
    Derate,
    Stop,
}

pub fn safe_step(prev: &Residual, next: &Residual) -> CorridorDecision {
    // Hard breach: any rx >= 1.0 ⇒ Stop.
    if next.coords.iter().any(|c| c.value >= 1.0) {
        return CorridorDecision::Stop;
    }

    // If any coord is outside its safe interior, require non‑increasing Vt.
    let any_outside_safe = next
        .coords
        .iter()
        .any(|c| c.value > c.bands.safe + 1e-9);

    if any_outside_safe && next.vt > prev.vt + 1e-9 {
        return CorridorDecision::Derate;
    }

    CorridorDecision::Ok
}

pub fn phoenix_default_corridors() -> Vec<CorridorBands> {
    vec![
        CorridorBands {
            var_id: "r_cpu",
            safe: 0.05,
            gold: 0.10,
            hard: 0.30,
            weight: 0.4,
            lyap_channel: 0,
        },
        CorridorBands {
            var_id: "r_ram",
            safe: 0.05,
            gold: 0.15,
            hard: 0.40,
            weight: 0.4,
            lyap_channel: 0,
        },
        CorridorBands {
            var_id: "r_net",
            safe: 0.02,
            gold: 0.10,
            hard: 0.30,
            weight: 0.2,
            lyap_channel: 0,
        },
    ]
}
