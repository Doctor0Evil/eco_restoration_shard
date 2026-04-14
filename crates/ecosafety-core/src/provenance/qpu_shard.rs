//! QPU Shard serialization with cryptographic evidencehex.
//! Implements ProvenanceKernel ALN spec for canonical hashing.

use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};
use ecosafety_core::{EvidenceHex, SignatureHex, UnixMillis, NodeId, RiskCoord, Lane};

/// QPU Shard V1 - the immutable data container.
/// Every mutable field contributes to evidencehex via canonical serialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QPUShardV1 {
    pub shard_id: EvidenceHex,
    pub prev_shard_id: EvidenceHex,
    pub aln_family: String,
    pub aln_version: String,
    pub lane: Lane,
    pub ker_k: f32,
    pub ker_e: f32,
    pub ker_r: f32,
    pub vt: f32,
    pub evidencehex: EvidenceHex,
    pub signinghex: Option<SignatureHex>,
    // Payload (variant based on aln_family)
    pub payload: ShardPayload,
}

/// Payload variants. For EcoSafetyRiskVector family.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShardPayload {
    RiskVectorV2 {
        r_energy: RiskCoord,
        r_hydraulic: RiskCoord,
        r_biology: RiskCoord,
        r_carbon: RiskCoord,
        r_materials: RiskCoord,
        r_dataquality: RiskCoord,
        r_sigma: RiskCoord,
        timestamp: UnixMillis,
        node_id: NodeId,
    },
    // Other families...
}

impl QPUShardV1 {
    /// Compute evidencehex from mutable fields in canonical order.
    /// Excludes shard_id, prev_shard_id, evidencehex, signinghex.
    pub fn compute_evidencehex(&self) -> EvidenceHex {
        let mut hasher = Sha256::new();
        self.canonical_write(&mut hasher);
        let result = hasher.finalize();
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&result);
        EvidenceHex(bytes)
    }

    /// Write canonical representation to a hasher or writer.
    fn canonical_write<W: std::io::Write>(&self, w: &mut W) {
        // Order defined by ProvenanceKernel ALN spec:
        // aln_family, aln_version, lane, ker_k, ker_e, ker_r, vt,
        // then payload fields in defined order.
        write!(w, "{}|", self.aln_family).unwrap();
        write!(w, "{}|", self.aln_version).unwrap();
        write!(w, "{}|", self.lane as u8).unwrap();
        write!(w, "{:.6}|", self.ker_k).unwrap();
        write!(w, "{:.6}|", self.ker_e).unwrap();
        write!(w, "{:.6}|", self.ker_r).unwrap();
        write!(w, "{:.6}|", self.vt).unwrap();
        match &self.payload {
            ShardPayload::RiskVectorV2 {
                r_energy, r_hydraulic, r_biology, r_carbon, r_materials,
                r_dataquality, r_sigma, timestamp, node_id
            } => {
                write!(w, "{:.6}|", r_energy.0).unwrap();
                write!(w, "{:.6}|", r_hydraulic.0).unwrap();
                write!(w, "{:.6}|", r_biology.0).unwrap();
                write!(w, "{:.6}|", r_carbon.0).unwrap();
                write!(w, "{:.6}|", r_materials.0).unwrap();
                write!(w, "{:.6}|", r_dataquality.0).unwrap();
                write!(w, "{:.6}|", r_sigma.0).unwrap();
                write!(w, "{}|", timestamp).unwrap();
                write!(w, "{}", node_id).unwrap();
            }
        }
    }

    /// Seal the shard: compute evidencehex, set shard_id = hash(prev_shard_id | evidencehex).
    pub fn seal(&mut self) {
        self.evidencehex = self.compute_evidencehex();
        let mut hasher = Sha256::new();
        hasher.update(&self.prev_shard_id.0);
        hasher.update(&self.evidencehex.0);
        let result = hasher.finalize();
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&result);
        self.shard_id = EvidenceHex(bytes);
    }

    /// Verify that evidencehex matches recomputed value.
    pub fn verify_evidence(&self) -> bool {
        self.evidencehex == self.compute_evidencehex()
    }

    /// Verify chain link: shard_id == hash(prev_shard_id | evidencehex).
    pub fn verify_chain(&self, prev_shard: Option<&QPUShardV1>) -> bool {
        if let Some(prev) = prev_shard {
            if self.prev_shard_id != prev.shard_id {
                return false;
            }
        }
        let mut hasher = Sha256::new();
        hasher.update(&self.prev_shard_id.0);
        hasher.update(&self.evidencehex.0);
        let result = hasher.finalize();
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&result);
        bytes == self.shard_id.0
    }
}

/// Create a new shard from validated risk vector.
impl QPUShardV1 {
    pub fn new_risk_vector(
        prev_shard_id: EvidenceHex,
        r_energy: RiskCoord,
        r_hydraulic: RiskCoord,
        r_biology: RiskCoord,
        r_carbon: RiskCoord,
        r_materials: RiskCoord,
        r_dataquality: RiskCoord,
        r_sigma: RiskCoord,
        timestamp: UnixMillis,
        node_id: NodeId,
        ker_k: f32,
        ker_e: f32,
        ker_r: f32,
        vt: f32,
        lane: Lane,
    ) -> Self {
        let mut shard = Self {
            shard_id: EvidenceHex([0; 32]), // placeholder
            prev_shard_id,
            aln_family: "EcoSafetyRiskVector".to_string(),
            aln_version: "2.0.0".to_string(),
            lane,
            ker_k,
            ker_e,
            ker_r,
            vt,
            evidencehex: EvidenceHex([0; 32]), // placeholder
            signinghex: None,
            payload: ShardPayload::RiskVectorV2 {
                r_energy, r_hydraulic, r_biology, r_carbon, r_materials,
                r_dataquality, r_sigma, timestamp, node_id,
            },
        };
        shard.seal();
        shard
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shard_evidence_integrity() {
        let mut shard = QPUShardV1::new_risk_vector(
            EvidenceHex([0; 32]),
            RiskCoord(0.1), RiskCoord(0.2), RiskCoord(0.3), RiskCoord(0.4),
            RiskCoord(0.5), RiskCoord(0.6), RiskCoord(0.7),
            1234567890000,
            "did:bostrom:node123".to_string(),
            0.95, 0.91, 0.12, 0.25,
            Lane::PILOT,
        );
        assert!(shard.verify_evidence());
        // Tamper with a field
        match &mut shard.payload {
            ShardPayload::RiskVectorV2 { r_energy, .. } => r_energy.0 = 0.99,
        }
        assert!(!shard.verify_evidence());
    }
}
