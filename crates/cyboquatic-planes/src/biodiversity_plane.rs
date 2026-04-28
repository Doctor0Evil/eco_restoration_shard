// File: crates/cyboquatic-planes/src/biodiversity_plane.rs

use serde::{Deserialize, Serialize};
use crate::types::RiskCoord;

/// Raw habitat metrics from modeling/measurement.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct BiodiversityRaw {
    /// Dimensionless connectivity index (e.g. 0–1 graph/circuit metric).
    pub connectivity_index: f64,
    /// Structural complexity (e.g. fractal dimension approx. 1–3).
    pub structural_complexity: f64,
    /// Colonization score (e.g. cover of target taxa).
    pub colonization_score: f64,
}

/// Corridors for each biodiversity dimension (higher is better).
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct BiodiversityCorridors {
    pub conn_gold: f64,
    pub conn_hard: f64,
    pub comp_gold: f64,
    pub comp_hard: f64,
    pub colon_gold: f64,
    pub colon_hard: f64,
    /// Aggregation weights.
    pub w_conn: f64,
    pub w_comp: f64,
    pub w_colon: f64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct BiodiversityScore {
    pub r_biodiversity: RiskCoord,
    pub r_conn: RiskCoord,
    pub r_comp: RiskCoord,
    pub r_colon: RiskCoord,
    pub corridor_ok: bool,
}

impl BiodiversityCorridors {
    /// Normalize "higher is better" metric into risk in [0,1].
    /// Values ≥ gold → low risk (~0); below hard → high risk (~1).
    fn normalize_inverse_good(gold: f64, hard: f64, value: f64) -> RiskCoord {
        let lo = hard;
        let hi = gold;
        let r = if value >= hi {
            0.0
        } else if value <= lo {
            1.0
        } else {
            (hi - value) / (hi - lo)
        };
        RiskCoord::new_clamped(r)
    }

    pub fn score(self, raw: BiodiversityRaw) -> BiodiversityScore {
        let r_conn = Self::normalize_inverse_good(self.conn_gold, self.conn_hard, raw.connectivity_index);
        let r_comp = Self::normalize_inverse_good(self.comp_gold, self.comp_hard, raw.structural_complexity);
        let r_colon = Self::normalize_inverse_good(self.colon_gold, self.colon_hard, raw.colonization_score);

        let w_sum = self.w_conn + self.w_comp + self.w_colon;
        let (w_conn, w_comp, w_colon) = if w_sum > 0.0 {
            (
                self.w_conn / w_sum,
                self.w_comp / w_sum,
                self.w_colon / w_sum,
            )
        } else {
            (1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0)
        };

        let r_bio_sq =
            w_conn * r_conn.value().powi(2) +
            w_comp * r_comp.value().powi(2) +
            w_colon * r_colon.value().powi(2);

        let r_biodiversity = RiskCoord::new_clamped(r_bio_sq.sqrt());

        let corridor_ok = raw.connectivity_index >= self.conn_hard - 1e-9
            && raw.structural_complexity >= self.comp_hard - 1e-9
            && raw.colonization_score >= self.colon_hard - 1e-9;

        BiodiversityScore {
            r_biodiversity,
            r_conn,
            r_comp,
            r_colon,
            corridor_ok,
        }
    }
}
