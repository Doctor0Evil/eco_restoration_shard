//! Corridor bands module - defines the grammar spine for normalized risk coordinates

use serde::{Deserialize, Serialize};

/// CorridorBands encodes the corridor bands, units, and Lyapunov weights for a given variable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorridorBands {
    pub varid: String,
    pub units: String,
    pub safe: f64,
    pub gold: f64,
    pub hard: f64,
    pub weight: f64,
    pub lyap_channel: String,
    pub mandatory: bool,
}

impl CorridorBands {
    /// Normalize a coordinate x to [0,1] using piecewise linear mapping.
    /// 
    /// - If x <= safe: returns 0.0 (fully safe)
    /// - If safe < x <= gold: returns (x - safe) / (gold - safe) * 0.5
    /// - If gold < x <= hard: returns 0.5 + (x - gold) / (hard - gold) * 0.5
    /// - If x > hard: returns 1.0 (fully violated)
    pub fn normalize_coord(&self, x: f64) -> f64 {
        if x <= self.safe {
            0.0
        } else if x <= self.gold {
            let range = self.gold - self.safe;
            if range <= 0.0 {
                return 0.5;
            }
            0.5 * (x - self.safe) / range
        } else if x <= self.hard {
            let range = self.hard - self.gold;
            if range <= 0.0 {
                return 1.0;
            }
            0.5 + 0.5 * (x - self.gold) / range
        } else {
            1.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_safe() {
        let bands = CorridorBands {
            varid: "rtox".to_string(),
            units: "normalized".to_string(),
            safe: 0.3,
            gold: 0.6,
            hard: 0.9,
            weight: 1.0,
            lyap_channel: "toxicity".to_string(),
            mandatory: true,
        };
        assert_eq!(bands.normalize_coord(0.2), 0.0);
        assert_eq!(bands.normalize_coord(0.3), 0.0);
    }

    #[test]
    fn test_normalize_gold() {
        let bands = CorridorBands {
            varid: "rtox".to_string(),
            units: "normalized".to_string(),
            safe: 0.3,
            gold: 0.6,
            hard: 0.9,
            weight: 1.0,
            lyap_channel: "toxicity".to_string(),
            mandatory: true,
        };
        let mid = bands.normalize_coord(0.45);
        assert!(mid > 0.0 && mid < 0.5);
    }

    #[test]
    fn test_normalize_hard() {
        let bands = CorridorBands {
            varid: "rtox".to_string(),
            units: "normalized".to_string(),
            safe: 0.3,
            gold: 0.6,
            hard: 0.9,
            weight: 1.0,
            lyap_channel: "toxicity".to_string(),
            mandatory: true,
        };
        assert_eq!(bands.normalize_coord(1.0), 1.0);
        assert_eq!(bands.normalize_coord(0.9), 1.0);
    }
}
