// File: crates/cyboquatic-planes/src/hydraulics_plane.rs

use serde::{Deserialize, Serialize};
use crate::types::RiskCoord;

/// Snapshot of hydraulic loading for a unit-process or reach.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct HydraulicsRaw {
    /// Hydraulic Loading Ratio (dimensionless load/capacity).
    pub hlr: f64,
}

/// Corridor bands for hydraulic stress.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct HydraulicsCorridor {
    /// Safe band (e.g. 0.7 × design capacity).
    pub hlr_safe: f64,
    /// Hard limit (onset of overflow/failure).
    pub hlr_hard: f64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct HydraulicsScore {
    pub r_hydraulics: RiskCoord,
    pub corridor_ok: bool,
}

impl HydraulicsCorridor {
    pub fn score(self, raw: HydraulicsRaw) -> HydraulicsScore {
        let hlr = raw.hlr.max(0.0);
        let lo = self.hlr_safe;
        let hi = self.hlr_hard.max(lo + 1e-6);

        let r = if hlr <= lo {
            0.0
        } else if hlr >= hi {
            1.0
        } else {
            (hlr - lo) / (hi - lo)
        };

        let corridor_ok = hlr <= hi + 1e-9;

        HydraulicsScore {
            r_hydraulics: RiskCoord::new_clamped(r),
            corridor_ok,
        }
    }
}
