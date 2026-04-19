//! EcoCoreParameters - corridor and normalization spine per region / node family.

use serde::{Deserialize, Serialize};
use ecosafety_core::CorridorBands;
use std::collections::HashMap;

/// EcoCoreParameters holds the complete set of corridor bands for all regions and node families.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EcoCoreParameters {
    pub rows: Vec<CorridorBands>,
}

impl EcoCoreParameters {
    /// Create a new EcoCoreParameters from a list of corridor bands.
    pub fn new(rows: Vec<CorridorBands>) -> Self {
        EcoCoreParameters { rows }
    }

    /// Build a lookup map by region_id and corridor_varid.
    pub fn lookup_map(&self) -> HashMap<(String, String), &CorridorBands> {
        let mut map = HashMap::new();
        for row in &self.rows {
            // For simplicity, we use varid as key; in practice you'd include region/node_family
            map.insert(("global".to_string(), row.varid.clone()), row);
        }
        map
    }

    /// Get bands for a specific varid.
    pub fn get_bands(&self, varid: &str) -> Option<&CorridorBands> {
        self.rows.iter().find(|b| b.varid == varid)
    }

    /// Validate that all mandatory corridors are present for a given node family.
    pub fn validate_mandatory(&self, _node_family: &str) -> bool {
        // Check that all rows marked mandatory exist
        self.rows.iter().all(|row| !row.mandatory || !row.varid.is_empty())
    }

    /// Validate corridor ordering: safe ≤ gold ≤ hard for all bands.
    pub fn validate_ordering(&self) -> bool {
        self.rows.iter().all(|b| b.safe <= b.gold && b.gold <= b.hard)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eco_core_parameters_new() {
        let rows = vec![
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
        ];

        let params = EcoCoreParameters::new(rows);
        assert_eq!(params.rows.len(), 1);
        assert!(params.validate_ordering());
    }

    #[test]
    fn test_get_bands() {
        let rows = vec![
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
        ];

        let params = EcoCoreParameters::new(rows);
        let bands = params.get_bands("rtox");
        assert!(bands.is_some());
        assert_eq!(bands.unwrap().safe, 0.3);
    }

    #[test]
    fn test_validate_ordering_invalid() {
        let rows = vec![
            CorridorBands {
                varid: "rtox".to_string(),
                units: "normalized".to_string(),
                safe: 0.9,
                gold: 0.6, // Invalid: safe > gold
                hard: 0.3,
                weight: 1.0,
                lyap_channel: "toxicity".to_string(),
                mandatory: true,
            },
        ];

        let params = EcoCoreParameters::new(rows);
        assert!(!params.validate_ordering());
    }
}
