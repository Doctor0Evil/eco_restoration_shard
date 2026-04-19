// crates/ecosafety-core/src/corridor.rs
//! CorridorBands typestate builder and measurement normalization.
//!
//! This module hardens the corridor grammar so any code that reaches
//! the kernels has guaranteed safe/gold/hard bands and monotone normalization.

use core::marker::PhantomData;

/// Typestate marker for missing corridor band.
#[derive(Clone, Copy, Debug)]
pub struct Missing;

/// Typestate marker for present corridor band.
#[derive(Clone, Copy, Debug)]
pub struct Present;

/// Corridor bands with typestate enforcement for complete initialization.
///
/// The type parameters S, G, H track whether safe, gold, and hard bands
/// have been set, respectively. Only `CorridorBands<Present, Present, Present>`
/// can be built into a usable corridor.
#[derive(Clone, Copy, Debug)]
pub struct CorridorBands<S, G, H> {
    pub safe: f64,
    pub gold: f64,
    pub hard: f64,
    _s: PhantomData<S>,
    _g: PhantomData<G>,
    _h: PhantomData<H>,
}

/// Incomplete corridor bands (nothing set).
pub type CorridorBandsIncomplete = CorridorBands<Missing, Missing, Missing>;

/// Complete corridor bands (all three bands set).
pub type CorridorBandsComplete = CorridorBands<Present, Present, Present>;

/// Builder for CorridorBands with typestate transitions.
pub struct CorridorBandsBuilder<S, G, H>(CorridorBands<S, G, H>);

impl CorridorBandsBuilder<Missing, Missing, Missing> {
    /// Create a new incomplete corridor bands builder.
    pub fn new() -> Self {
        Self(CorridorBands {
            safe: 0.0,
            gold: 0.0,
            hard: 0.0,
            _s: PhantomData,
            _g: PhantomData,
            _h: PhantomData,
        })
    }
}

impl Default for CorridorBandsBuilder<Missing, Missing, Missing> {
    fn default() -> Self {
        Self::new()
    }
}

impl<G, H> CorridorBandsBuilder<Missing, G, H> {
    /// Set the safe band value.
    pub fn with_safe(self, v: f64) -> CorridorBandsBuilder<Present, G, H> {
        CorridorBandsBuilder(CorridorBands {
            safe: v,
            gold: self.0.gold,
            hard: self.0.hard,
            _s: PhantomData,
            _g: PhantomData,
            _h: PhantomData,
        })
    }
}

impl<S, H> CorridorBandsBuilder<S, Missing, H> {
    /// Set the gold band value.
    pub fn with_gold(self, v: f64) -> CorridorBandsBuilder<S, Present, H> {
        CorridorBandsBuilder(CorridorBands {
            safe: self.0.safe,
            gold: v,
            hard: self.0.hard,
            _s: PhantomData,
            _g: PhantomData,
            _h: PhantomData,
        })
    }
}

impl<S, G> CorridorBandsBuilder<S, G, Missing> {
    /// Set the hard band value.
    pub fn with_hard(self, v: f64) -> CorridorBandsBuilder<S, G, Present> {
        CorridorBandsBuilder(CorridorBands {
            safe: self.0.safe,
            gold: self.0.gold,
            hard: v,
            _s: PhantomData,
            _g: PhantomData,
            _h: PhantomData,
        })
    }
}

impl CorridorBandsBuilder<Present, Present, Present> {
    /// Build the complete corridor bands.
    ///
    /// # Panics
    /// Panics if the bands are not monotonically increasing (safe < gold < hard).
    pub fn build(self) -> CorridorBandsComplete {
        assert!(
            self.0.safe < self.0.gold && self.0.gold < self.0.hard,
            "Corridor bands must be monotonically increasing: safe ({}) < gold ({}) < hard ({})",
            self.0.safe,
            self.0.gold,
            self.0.hard
        );
        self.0
    }

    /// Build without checking monotonicity (for advanced use cases).
    pub fn build_unchecked(self) -> CorridorBandsComplete {
        self.0
    }
}

/// A normalized risk coordinate in [0, 1].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RiskCoord {
    pub r: f64,      // Normalized risk in [0, 1]
    pub sigma: f64,  // Uncertainty estimate
}

impl RiskCoord {
    /// Create a new risk coordinate.
    ///
    /// # Panics
    /// Panics if r is not in [0, 1].
    pub fn new(r: f64, sigma: f64) -> Self {
        assert!(
            (0.0..=1.0).contains(&r),
            "Risk coordinate {} is not in [0, 1]",
            r
        );
        Self { r, sigma }
    }

    /// Create a new risk coordinate without bounds checking.
    pub fn new_unchecked(r: f64, sigma: f64) -> Self {
        Self { r, sigma }
    }
}

/// Error types for normalization operations.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NormalizationError {
    /// Measurement value is physically impossible (e.g., negative concentration).
    PhysicallyImpossible { value: f64, reason: &'static str },
    /// Corridor bands are incomplete (should be caught at compile time with typestates).
    CorridorIncomplete,
    /// Measurement is outside the corridor range entirely.
    OutOfCorridor { value: f64, corridor_max: f64 },
    /// Weight is negative.
    NegativeWeight { weight: f64 },
}

impl core::fmt::Display for NormalizationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            NormalizationError::PhysicallyImpossible { value, reason } => {
                write!(f, "physically impossible value {}: {}", value, reason)
            }
            NormalizationError::CorridorIncomplete => {
                write!(f, "corridor bands are incomplete")
            }
            NormalizationError::OutOfCorridor { value, corridor_max } => {
                write!(f, "value {} exceeds corridor maximum {}", value, corridor_max)
            }
            NormalizationError::NegativeWeight { weight } => {
                write!(f, "negative weight {}", weight)
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for NormalizationError {}

/// Normalize a measurement to a risk coordinate in [0, 1].
///
/// Uses piecewise linear normalization:
/// - Values <= safe map to r = 0
/// - Values between safe and gold map to r in (0, 0.5]
/// - Values between gold and hard map to r in (0.5, 1]
/// - Values > hard are clamped to r = 1 (or return error with strict mode)
///
/// # Arguments
/// * `value` - The raw measurement value
/// * `corridor` - Complete corridor bands
/// * `sigma` - Uncertainty estimate for the measurement
/// * `strict` - If true, return error for values > hard; if false, clamp to 1.0
///
/// # Returns
/// * `Ok(RiskCoord)` - Normalized risk coordinate
/// * `Err(NormalizationError)` - Reason normalization failed
pub fn normalize_measurement(
    value: f64,
    corridor: &CorridorBandsComplete,
    sigma: f64,
    strict: bool,
) -> Result<RiskCoord, NormalizationError> {
    // Check for physically impossible values (example: negative concentrations)
    if value < 0.0 {
        return Err(NormalizationError::PhysicallyImpossible {
            value,
            reason: "negative measurement",
        });
    }

    // Compute normalized risk coordinate
    let r = if value <= corridor.safe {
        0.0
    } else if value <= corridor.gold {
        // Linear interpolation from 0 to 0.5
        0.5 * (value - corridor.safe) / (corridor.gold - corridor.safe)
    } else if value <= corridor.hard {
        // Linear interpolation from 0.5 to 1.0
        0.5 + 0.5 * (value - corridor.gold) / (corridor.hard - corridor.gold)
    } else {
        // Value exceeds hard limit
        if strict {
            return Err(NormalizationError::OutOfCorridor {
                value,
                corridor_max: corridor.hard,
            });
        }
        1.0 // Clamp
    };

    Ok(RiskCoord::new(r, sigma))
}

/// Normalize a measurement with automatic sigma estimation based on distance from safe.
///
/// Sigma increases as the value approaches hard limit, reflecting higher uncertainty
/// near boundary conditions.
pub fn normalize_measurement_auto_sigma(
    value: f64,
    corridor: &CorridorBandsComplete,
    strict: bool,
) -> Result<RiskCoord, NormalizationError> {
    // Estimate sigma: 0.01 at safe, 0.1 at hard
    let base_sigma = 0.01;
    let max_sigma = 0.1;
    
    let sigma = if value <= corridor.safe {
        base_sigma
    } else if value >= corridor.hard {
        max_sigma
    } else {
        base_sigma + (max_sigma - base_sigma) * 
            (value - corridor.safe) / (corridor.hard - corridor.safe)
    };

    normalize_measurement(value, corridor, sigma, strict)
}

/// Table of corridor bands for multiple variables.
#[derive(Clone, Debug)]
pub struct CorridorTable {
    pub bands: Vec<(String, CorridorBandsComplete)>,
}

impl CorridorTable {
    pub fn new() -> Self {
        Self { bands: Vec::new() }
    }

    pub fn add(&mut self, var_id: String, bands: CorridorBandsComplete) {
        self.bands.push((var_id, bands));
    }

    pub fn get(&self, var_id: &str) -> Option<&CorridorBandsComplete> {
        self.bands
            .iter()
            .find(|(id, _)| id == var_id)
            .map(|(_, bands)| bands)
    }
}

impl Default for CorridorTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_corridor_builder_complete() {
        let corridor = CorridorBandsBuilder::new()
            .with_safe(10.0)
            .with_gold(50.0)
            .with_hard(100.0)
            .build();

        assert_eq!(corridor.safe, 10.0);
        assert_eq!(corridor.gold, 50.0);
        assert_eq!(corridor.hard, 100.0);
    }

    #[test]
    #[should_panic(expected = "monotonically increasing")]
    fn test_corridor_builder_non_monotonic() {
        CorridorBandsBuilder::new()
            .with_safe(50.0)
            .with_gold(10.0)  // Wrong order
            .with_hard(100.0)
            .build();
    }

    #[test]
    fn test_normalize_measurement_safe() {
        let corridor = CorridorBandsBuilder::new()
            .with_safe(10.0)
            .with_gold(50.0)
            .with_hard(100.0)
            .build();

        let result = normalize_measurement(5.0, &corridor, 0.01, false).unwrap();
        assert_eq!(result.r, 0.0);
    }

    #[test]
    fn test_normalize_measurement_gold() {
        let corridor = CorridorBandsBuilder::new()
            .with_safe(10.0)
            .with_gold(50.0)
            .with_hard(100.0)
            .build();

        let result = normalize_measurement(30.0, &corridor, 0.05, false).unwrap();
        assert!((result.r - 0.25).abs() < 1e-10); // Midpoint between safe and gold
    }

    #[test]
    fn test_normalize_measurement_hard() {
        let corridor = CorridorBandsBuilder::new()
            .with_safe(10.0)
            .with_gold(50.0)
            .with_hard(100.0)
            .build();

        let result = normalize_measurement(75.0, &corridor, 0.08, false).unwrap();
        assert!((result.r - 0.75).abs() < 1e-10); // Midpoint between gold and hard
    }

    #[test]
    fn test_normalize_measurement_exceeds_strict() {
        let corridor = CorridorBandsBuilder::new()
            .with_safe(10.0)
            .with_gold(50.0)
            .with_hard(100.0)
            .build();

        let result = normalize_measurement(150.0, &corridor, 0.1, true);
        assert!(matches!(result, Err(NormalizationError::OutOfCorridor { .. })));
    }

    #[test]
    fn test_normalize_measurement_exceeds_clamped() {
        let corridor = CorridorBandsBuilder::new()
            .with_safe(10.0)
            .with_gold(50.0)
            .with_hard(100.0)
            .build();

        let result = normalize_measurement(150.0, &corridor, 0.1, false).unwrap();
        assert_eq!(result.r, 1.0);
    }

    #[test]
    fn test_normalize_measurement_negative() {
        let corridor = CorridorBandsBuilder::new()
            .with_safe(10.0)
            .with_gold(50.0)
            .with_hard(100.0)
            .build();

        let result = normalize_measurement(-5.0, &corridor, 0.01, false);
        assert!(matches!(result, Err(NormalizationError::PhysicallyImpossible { .. })));
    }

    #[test]
    fn test_corridor_table() {
        let mut table = CorridorTable::new();
        
        let corridor1 = CorridorBandsBuilder::new()
            .with_safe(10.0)
            .with_gold(50.0)
            .with_hard(100.0)
            .build();
        
        let corridor2 = CorridorBandsBuilder::new()
            .with_safe(0.0)
            .with_gold(0.5)
            .with_hard(1.0)
            .build();

        table.add("temperature".to_string(), corridor1);
        table.add("ph".to_string(), corridor2);

        assert!(table.get("temperature").is_some());
        assert!(table.get("ph").is_some());
        assert!(table.get("humidity").is_none());
    }
}
