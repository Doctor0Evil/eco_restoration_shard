//! Residual kernel module - computes V_t residual from normalized coordinates

use serde::{Deserialize, Serialize};
use crate::corridor::CorridorBands;
use std::collections::HashMap;

/// Residual represents the Lyapunov-like residual V_t and its contributing coordinates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Residual {
    pub vt: f64,
    pub coords: Vec<(String, f64)>,
}

/// Compute the residual V_t from normalized coordinates and corridor bands.
/// 
/// V_t = sum_i(w_i * r_i^2) where w_i is the weight from CorridorBands and r_i is the normalized coordinate.
pub fn compute_residual(
    coords: &[(String, f64)],
    bands: &HashMap<String, CorridorBands>,
) -> Residual {
    let mut vt = 0.0;
    let mut normalized_coords = Vec::new();

    for (varid, value) in coords {
        let norm_value = if let Some(band) = bands.get(varid) {
            band.normalize_coord(*value)
        } else {
            // If no band exists, treat as fully violated (conservative)
            1.0
        };

        let weight = bands.get(varid).map(|b| b.weight).unwrap_or(1.0);
        vt += weight * norm_value * norm_value;
        normalized_coords.push((varid.clone(), norm_value));
    }

    Residual {
        vt,
        coords: normalized_coords,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_residual_empty() {
        let bands: HashMap<String, CorridorBands> = HashMap::new();
        let res = compute_residual(&[], &bands);
        assert_eq!(res.vt, 0.0);
    }

    #[test]
    fn test_compute_residual_single() {
        let mut bands = HashMap::new();
        bands.insert(
            "rtox".to_string(),
            CorridorBands {
                varid: "rtox".to_string(),
                units: "normalized".to_string(),
                safe: 0.3,
                gold: 0.6,
                hard: 0.9,
                weight: 1.0,
                lyap_channel: "toxicity".to_string(),
                mandatory: true,
            },
        );

        let coords = vec![("rtox".to_string(), 0.2)]; // Fully safe
        let res = compute_residual(&coords, &bands);
        assert_eq!(res.vt, 0.0);

        let coords = vec![("rtox".to_string(), 1.0)]; // Fully violated
        let res = compute_residual(&coords, &bands);
        assert_eq!(res.vt, 1.0);
    }
}
