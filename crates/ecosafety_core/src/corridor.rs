//! Corridor bands module - defines the grammar spine for normalized risk coordinates
//! 
//! Implements typestate builder pattern to ensure safe/gold/hard bands are always complete
//! before use in kernels.

use serde::{Deserialize, Serialize};
use core::marker::PhantomData;

/// Typestate marker for missing band value
pub struct Missing;
/// Typestate marker for present band value
pub struct Present;

/// CorridorBands encodes the corridor bands, units, and Lyapunov weights for a given variable.
/// Uses typestate pattern to enforce complete initialization at compile time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorridorBands<S, G, H> {
    pub varid: String,
    pub units: String,
    pub safe: f64,
    pub gold: f64,
    pub hard: f64,
    pub weight: f64,
    pub lyap_channel: String,
    pub mandatory: bool,
    _s: PhantomData<S>,
    _g: PhantomData<G>,
    _h: PhantomData<H>,
}

/// Type alias for incomplete corridor bands (missing all three thresholds)
pub type CorridorBandsIncomplete = CorridorBands<Missing, Missing, Missing>;
/// Type alias for complete corridor bands (all thresholds present)
pub type CorridorBandsComplete = CorridorBands<Present, Present, Present>;

/// Builder for CorridorBands with typestate enforcement
pub struct CorridorBandsBuilder<S, G, H>(CorridorBands<S, G, H>);

impl CorridorBandsBuilder<Missing, Missing, Missing> {
    /// Create a new incomplete CorridorBands builder
    pub fn new(varid: String, units: String, weight: f64, lyap_channel: String, mandatory: bool) -> Self {
        Self(CorridorBands {
            varid,
            units,
            safe: 0.0,
            gold: 0.0,
            hard: 0.0,
            weight,
            lyap_channel,
            mandatory,
            _s: PhantomData,
            _g: PhantomData,
            _h: PhantomData,
        })
    }
}

impl<G, H> CorridorBandsBuilder<Missing, G, H> {
    /// Set the safe threshold
    pub fn with_safe(self, v: f64) -> CorridorBandsBuilder<Present, G, H> {
        let mut b = self.0;
        b.safe = v;
        CorridorBandsBuilder(b)
    }
}

impl<S, H> CorridorBandsBuilder<S, Missing, H> {
    /// Set the gold threshold
    pub fn with_gold(self, v: f64) -> CorridorBandsBuilder<S, Present, H> {
        let mut b = self.0;
        b.gold = v;
        CorridorBandsBuilder(b)
    }
}

impl<S, G> CorridorBandsBuilder<S, G, Missing> {
    /// Set the hard threshold
    pub fn with_hard(self, v: f64) -> CorridorBandsBuilder<S, G, Present> {
        let mut b = self.0;
        b.hard = v;
        CorridorBandsBuilder(b)
    }
}

impl CorridorBandsBuilder<Present, Present, Present> {
    /// Build the complete CorridorBands
    pub fn build(self) -> CorridorBandsComplete {
        CorridorBands {
            varid: self.0.varid,
            units: self.0.units,
            safe: self.0.safe,
            gold: self.0.gold,
            hard: self.0.hard,
            weight: self.0.weight,
            lyap_channel: self.0.lyap_channel,
            mandatory: self.0.mandatory,
            _s: PhantomData,
            _g: PhantomData,
            _h: PhantomData,
        }
    }
}

// Legacy constructor for backward compatibility
impl CorridorBands<Present, Present, Present> {
    /// Create a complete CorridorBands directly (legacy API)
    pub fn new_legacy(
        varid: String,
        units: String,
        safe: f64,
        gold: f64,
        hard: f64,
        weight: f64,
        lyap_channel: String,
        mandatory: bool,
    ) -> Self {
        Self {
            varid,
            units,
            safe,
            gold,
            hard,
            weight,
            lyap_channel,
            mandatory,
            _s: PhantomData,
            _g: PhantomData,
            _h: PhantomData,
        }
    }
}

impl CorridorBands<Present, Present, Present> {
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
    
    /// Validate that thresholds are monotonically increasing
    pub fn validate_thresholds(&self) -> Result<(), &'static str> {
        if self.safe > self.gold || self.gold > self.hard {
            return Err("Thresholds must be monotonically increasing: safe <= gold <= hard");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typestate_builder() {
        // This should compile: complete builder chain
        let bands: CorridorBandsComplete = CorridorBandsBuilder::new(
            "rtox".to_string(),
            "normalized".to_string(),
            1.0,
            "toxicity".to_string(),
            true,
        )
        .with_safe(0.3)
        .with_gold(0.6)
        .with_hard(0.9)
        .build();
        
        assert_eq!(bands.safe, 0.3);
        assert_eq!(bands.gold, 0.6);
        assert_eq!(bands.hard, 0.9);
    }

    #[test]
    fn test_normalize_safe() {
        let bands = CorridorBands::<Present, Present, Present>::new_legacy(
            "rtox".to_string(),
            "normalized".to_string(),
            0.3,
            0.6,
            0.9,
            1.0,
            "toxicity".to_string(),
            true,
        );
        assert_eq!(bands.normalize_coord(0.2), 0.0);
        assert_eq!(bands.normalize_coord(0.3), 0.0);
    }

    #[test]
    fn test_normalize_gold() {
        let bands = CorridorBands::<Present, Present, Present>::new_legacy(
            "rtox".to_string(),
            "normalized".to_string(),
            0.3,
            0.6,
            0.9,
            1.0,
            "toxicity".to_string(),
            true,
        );
        let mid = bands.normalize_coord(0.45);
        assert!(mid > 0.0 && mid < 0.5);
    }

    #[test]
    fn test_normalize_hard() {
        let bands = CorridorBands::<Present, Present, Present>::new_legacy(
            "rtox".to_string(),
            "normalized".to_string(),
            0.3,
            0.6,
            0.9,
            1.0,
            "toxicity".to_string(),
            true,
        );
        assert_eq!(bands.normalize_coord(1.0), 1.0);
        assert_eq!(bands.normalize_coord(0.9), 1.0);
    }
    
    #[test]
    fn test_validate_thresholds() {
        let bands = CorridorBands::<Present, Present, Present>::new_legacy(
            "rtox".to_string(),
            "normalized".to_string(),
            0.3,
            0.6,
            0.9,
            1.0,
            "toxicity".to_string(),
            true,
        );
        assert!(bands.validate_thresholds().is_ok());
        
        let bad_bands = CorridorBands::<Present, Present, Present>::new_legacy(
            "rtox".to_string(),
            "normalized".to_string(),
            0.9,
            0.6,
            0.3,
            1.0,
            "toxicity".to_string(),
            true,
        );
        assert!(bad_bands.validate_thresholds().is_err());
    }
}
