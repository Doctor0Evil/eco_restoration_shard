//! Corridor band definitions and piecewise-affine normalization kernels.
//! This module implements the canonical normalization used by all ecosafety components.

use serde::{Deserialize, Serialize};
use ecosafety_core::RiskCoord;

/// Six thresholds defining corridor bands.
/// Order: [safe_low, gold_low, hard_low, hard_high, gold_high, safe_high]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SafeGoldHard {
    pub safe_low: f32,
    pub gold_low: f32,
    pub hard_low: f32,
    pub hard_high: f32,
    pub gold_high: f32,
    pub safe_high: f32,
}

impl SafeGoldHard {
    pub fn from_array(arr: [f32; 6]) -> Self {
        Self {
            safe_low: arr[0],
            gold_low: arr[1],
            hard_low: arr[2],
            hard_high: arr[3],
            gold_high: arr[4],
            safe_high: arr[5],
        }
    }

    pub fn to_array(&self) -> [f32; 6] {
        [
            self.safe_low, self.gold_low, self.hard_low,
            self.hard_high, self.gold_high, self.safe_high,
        ]
    }

    /// Validate that thresholds are monotonically increasing and within reasonable bounds.
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.safe_low > self.gold_low
            || self.gold_low > self.hard_low
            || self.hard_low > self.hard_high
            || self.hard_high > self.gold_high
            || self.gold_high > self.safe_high
        {
            return Err("Thresholds must be monotonically increasing");
        }
        Ok(())
    }
}

/// Normalization kind.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum NormKind {
    PiecewiseAffine,
    Logarithmic,
    Identity,
}

/// Complete corridor definition for a single risk coordinate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorridorBand {
    pub coord_id: String,
    pub safegoldhard: SafeGoldHard,
    pub weight: f32,
    pub normkind: NormKind,
    pub unit: Option<String>,
    pub description: Option<String>,
}

impl CorridorBand {
    /// Normalize a raw measurement to a RiskCoord in [0,1] using piecewise-affine mapping.
    /// This is the canonical kernel reused across all profiles.
    #[inline]
    pub fn normalize(&self, raw: f32) -> RiskCoord {
        match self.normkind {
            NormKind::PiecewiseAffine => self.normalize_piecewise(raw),
            NormKind::Logarithmic => self.normalize_log(raw),
            NormKind::Identity => RiskCoord(raw.clamp(0.0, 1.0)),
        }
    }

    fn normalize_piecewise(&self, raw: f32) -> RiskCoord {
        let s = self.safegoldhard;
        let r = if raw <= s.safe_low {
            0.0
        } else if raw <= s.gold_low {
            (raw - s.safe_low) / (s.gold_low - s.safe_low) * 0.25
        } else if raw <= s.hard_low {
            0.25 + (raw - s.gold_low) / (s.hard_low - s.gold_low) * 0.25
        } else if raw <= s.hard_high {
            0.5 + (raw - s.hard_low) / (s.hard_high - s.hard_low) * 0.25
        } else if raw <= s.gold_high {
            0.75 + (raw - s.hard_high) / (s.gold_high - s.hard_high) * 0.15
        } else if raw <= s.safe_high {
            0.9 + (raw - s.gold_high) / (s.safe_high - s.gold_high) * 0.1
        } else {
            1.0
        };
        RiskCoord(r.clamp(0.0, 1.0))
    }

    fn normalize_log(&self, raw: f32) -> RiskCoord {
        // Placeholder for logarithmic normalization (e.g., pH)
        // Implement according to ALN spec.
        RiskCoord(0.5)
    }
}

/// Set of corridors for all 7 canonical risk coordinates.
#[derive(Debug, Clone)]
pub struct CorridorSet {
    pub bands: [CorridorBand; 7],
}

impl CorridorSet {
    /// Load from ALN specification file.
    pub fn from_aln(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // In practice, use the procedural macro or a parser.
        // Here we provide a stub with default values from the ALN spec.
        Ok(Self::default())
    }

    /// Normalize an array of 7 raw measurements.
    pub fn normalize_all(&self, raw: &[f32; 7]) -> [RiskCoord; 7] {
        [
            self.bands[0].normalize(raw[0]),
            self.bands[1].normalize(raw[1]),
            self.bands[2].normalize(raw[2]),
            self.bands[3].normalize(raw[3]),
            self.bands[4].normalize(raw[4]),
            self.bands[5].normalize(raw[5]),
            self.bands[6].normalize(raw[6]),
        ]
    }

    /// Get weights array for Vt computation.
    pub fn weights(&self) -> [f32; 7] {
        self.bands.iter().map(|b| b.weight).collect::<Vec<_>>().try_into().unwrap()
    }

    /// Validate all bands (threshold monotonicity, weight sum approx 1.0).
    pub fn validate(&self) -> Result<(), String> {
        for band in &self.bands {
            band.safegoldhard.validate().map_err(|e| format!("{}: {}", band.coord_id, e))?;
        }
        let sum: f32 = self.bands.iter().map(|b| b.weight).sum();
        if (sum - 1.0).abs() > 0.001 {
            return Err(format!("Weights sum to {}, expected 1.0", sum));
        }
        Ok(())
    }
}

impl Default for CorridorSet {
    fn default() -> Self {
        Self {
            bands: [
                CorridorBand {
                    coord_id: "r_energy".to_string(),
                    safegoldhard: SafeGoldHard::from_array([0.0, 0.15, 0.35, 0.55, 0.85, 1.0]),
                    weight: 0.12,
                    normkind: NormKind::PiecewiseAffine,
                    unit: Some("kWh".to_string()),
                    description: Some("Energy consumption".to_string()),
                },
                CorridorBand {
                    coord_id: "r_hydraulic".to_string(),
                    safegoldhard: SafeGoldHard::from_array([0.0, 0.10, 0.25, 0.40, 0.70, 1.0]),
                    weight: 0.18,
                    normkind: NormKind::PiecewiseAffine,
                    unit: Some("m³/s".to_string()),
                    description: Some("Hydraulic load".to_string()),
                },
                CorridorBand {
                    coord_id: "r_biology".to_string(),
                    safegoldhard: SafeGoldHard::from_array([0.0, 0.08, 0.20, 0.35, 0.60, 1.0]),
                    weight: 0.20,
                    normkind: NormKind::PiecewiseAffine,
                    unit: Some("mg/L".to_string()),
                    description: Some("Dissolved oxygen".to_string()),
                },
                CorridorBand {
                    coord_id: "r_carbon".to_string(),
                    safegoldhard: SafeGoldHard::from_array([0.0, 0.12, 0.30, 0.50, 0.80, 1.0]),
                    weight: 0.15,
                    normkind: NormKind::PiecewiseAffine,
                    unit: Some("kg CO₂e".to_string()),
                    description: Some("Carbon intensity".to_string()),
                },
                CorridorBand {
                    coord_id: "r_materials".to_string(),
                    safegoldhard: SafeGoldHard::from_array([0.0, 0.10, 0.25, 0.45, 0.75, 1.0]),
                    weight: 0.15,
                    normkind: NormKind::PiecewiseAffine,
                    unit: Some("toxicity".to_string()),
                    description: Some("Material degradation".to_string()),
                },
                CorridorBand {
                    coord_id: "r_dataquality".to_string(),
                    safegoldhard: SafeGoldHard::from_array([0.0, 0.05, 0.15, 0.30, 0.60, 1.0]),
                    weight: 0.10,
                    normkind: NormKind::PiecewiseAffine,
                    unit: Some("%".to_string()),
                    description: Some("Data quality".to_string()),
                },
                CorridorBand {
                    coord_id: "r_sigma".to_string(),
                    safegoldhard: SafeGoldHard::from_array([0.0, 0.10, 0.25, 0.45, 0.70, 1.0]),
                    weight: 0.10,
                    normkind: NormKind::PiecewiseAffine,
                    unit: Some("unitless".to_string()),
                    description: Some("Model uncertainty".to_string()),
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn piecewise_normalization() {
        let band = CorridorBand {
            coord_id: "test".into(),
            safegoldhard: SafeGoldHard::from_array([0.0, 2.0, 4.0, 6.0, 8.0, 10.0]),
            weight: 0.1,
            normkind: NormKind::PiecewiseAffine,
            unit: None,
            description: None,
        };
        assert_eq!(band.normalize(-1.0).0, 0.0);
        assert_eq!(band.normalize(1.0).0, 0.125);
        assert_eq!(band.normalize(3.0).0, 0.375);
        assert_eq!(band.normalize(5.0).0, 0.625);
        assert_eq!(band.normalize(7.0).0, 0.825);
        assert_eq!(band.normalize(9.0).0, 0.95);
        assert_eq!(band.normalize(11.0).0, 1.0);
    }

    #[test]
    fn corridor_set_weights_sum() {
        let set = CorridorSet::default();
        assert!(set.validate().is_ok());
    }
}
