//! ResponseShardEcoTurn - turn-level "what just happened" shards for research, code, or ops steps.

use serde::{Deserialize, Serialize};
use ecosafety_core::{KerTriad, Residual, CorridorBands};
use std::collections::HashMap;

/// ResponseShardEcoTurn represents a single turn/step record in the shard ledger.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseShardEcoTurn {
    // Identity fields
    pub userdid: String,
    pub authordid: String,
    pub nodeid: String,
    pub region: String,
    pub topic: String,
    pub lane: String, // RESEARCH|PILOT|PROD
    
    // Time fields
    pub twindow_start_utc: String,
    pub twindow_end_utc: String,
    
    // KER triad
    pub knowledge_factor: f64,
    pub eco_impact: f64,
    pub risk_of_harm: f64,
    
    // Residual spine
    pub vt_before: f64,
    pub vt_after: f64,
    pub rx_map_json: String, // JSON map varid→rx
    
    // Progress hooks
    pub corridor_update_ids: String, // Comma-separated IDs
    pub equation_update_ids: String, // Comma-separated IDs
    
    // Governance
    pub ker_deployable: bool,
    pub kertarget_met: bool,
    pub promotion_reason: String,
    pub hexstamp: String,
}

impl ResponseShardEcoTurn {
    /// Create a new ResponseShardEcoTurn from parsed CSV row data.
    pub fn from_csv_row(row: &csv::StringRecord) -> Result<Self, csv::Error> {
        Ok(ResponseShardEcoTurn {
            userdid: row.get(0).unwrap_or("").to_string(),
            authordid: row.get(1).unwrap_or("").to_string(),
            nodeid: row.get(2).unwrap_or("").to_string(),
            region: row.get(3).unwrap_or("").to_string(),
            topic: row.get(4).unwrap_or("").to_string(),
            lane: row.get(5).unwrap_or("").to_string(),
            twindow_start_utc: row.get(6).unwrap_or("").to_string(),
            twindow_end_utc: row.get(7).unwrap_or("").to_string(),
            knowledge_factor: row.get(8).and_then(|s| s.parse().ok()).unwrap_or(0.0),
            eco_impact: row.get(9).and_then(|s| s.parse().ok()).unwrap_or(0.0),
            risk_of_harm: row.get(10).and_then(|s| s.parse().ok()).unwrap_or(1.0),
            vt_before: row.get(11).and_then(|s| s.parse().ok()).unwrap_or(1.0),
            vt_after: row.get(12).and_then(|s| s.parse().ok()).unwrap_or(1.0),
            rx_map_json: row.get(13).unwrap_or("{}").to_string(),
            corridor_update_ids: row.get(14).unwrap_or("").to_string(),
            equation_update_ids: row.get(15).unwrap_or("").to_string(),
            ker_deployable: row.get(16).and_then(|s| s.parse().ok()).unwrap_or(false),
            kertarget_met: row.get(17).and_then(|s| s.parse().ok()).unwrap_or(false),
            promotion_reason: row.get(18).unwrap_or("").to_string(),
            hexstamp: row.get(19).unwrap_or("").to_string(),
        })
    }

    /// Convert this shard to a CSV row string.
    pub fn to_csv_row(&self) -> String {
        format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            self.userdid, self.authordid, self.nodeid, self.region, self.topic, self.lane,
            self.twindow_start_utc, self.twindow_end_utc,
            self.knowledge_factor, self.eco_impact, self.risk_of_harm,
            self.vt_before, self.vt_after, self.rx_map_json,
            self.corridor_update_ids, self.equation_update_ids,
            self.ker_deployable, self.kertarget_met,
            self.promotion_reason, self.hexstamp
        )
    }

    /// Recompute and validate KER triad and deployability against core parameters.
    pub fn recompute_and_validate(&mut self, _core: &EcoCoreParameters) -> KerTriad {
        // Parse rx_map_json to get normalized coordinates
        let coords: Vec<(String, f64)> = serde_json::from_str(&self.rx_map_json)
            .unwrap_or_default();

        // Compute R from coordinates (simplified - would use actual weights from core)
        let r = if coords.is_empty() {
            1.0
        } else {
            coords.iter().map(|(_, v)| v).sum::<f64>() / coords.len() as f64
        };

        let ker = KerTriad {
            k: self.knowledge_factor,
            e: self.eco_impact,
            r,
        };

        // Update deployability flags
        self.ker_deployable = ker.k >= 0.9 && ker.e >= 0.9 && ker.r <= 0.13;
        self.kertarget_met = self.ker_deployable;

        ker
    }

    /// Check if this shard meets PROD lane invariants.
    pub fn is_prod_valid(&self) -> bool {
        if self.lane == "PROD" {
            self.ker_deployable && self.kertarget_met
        } else {
            true
        }
    }
}

/// EcoCoreParameters stub for now - full impl in eco_core_parameters module.
pub struct EcoCoreParameters {
    pub bands: HashMap<String, CorridorBands>,
}

impl Default for EcoCoreParameters {
    fn default() -> Self {
        EcoCoreParameters {
            bands: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_shard_from_csv() {
        let row = csv::StringRecord::from(vec![
            "did:user1", "did:author1", "node1", "region1", "test_topic", "RESEARCH",
            "2026-01-01T00:00:00Z", "2026-01-01T01:00:00Z",
            "0.9", "0.85", "0.1",
            "0.5", "0.4", "{}", "", "", "false", "false", "", ""
        ]);

        let shard = ResponseShardEcoTurn::from_csv_row(&row).unwrap();
        assert_eq!(shard.topic, "test_topic");
        assert!((shard.knowledge_factor - 0.9).abs() < 0.001);
    }

    #[test]
    fn test_prod_lane_invariant() {
        let mut shard = ResponseShardEcoTurn {
            userdid: "did:user1".to_string(),
            authordid: "did:author1".to_string(),
            nodeid: "node1".to_string(),
            region: "region1".to_string(),
            topic: "test".to_string(),
            lane: "PROD".to_string(),
            twindow_start_utc: "2026-01-01T00:00:00Z".to_string(),
            twindow_end_utc: "2026-01-01T01:00:00Z".to_string(),
            knowledge_factor: 0.95,
            eco_impact: 0.92,
            risk_of_harm: 0.10,
            vt_before: 0.5,
            vt_after: 0.4,
            rx_map_json: "{}".to_string(),
            corridor_update_ids: "".to_string(),
            equation_update_ids: "".to_string(),
            ker_deployable: true,
            kertarget_met: true,
            promotion_reason: "".to_string(),
            hexstamp: "abc123".to_string(),
        };

        assert!(shard.is_prod_valid());

        shard.ker_deployable = false;
        assert!(!shard.is_prod_valid());
    }
}
