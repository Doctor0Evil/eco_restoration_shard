//! Deploy decision workflow with Bostrom DID signing.
//! Produces cryptographically signed decisions that can be anchored on-chain.

use ecosafety_core::{
    EvidenceHex, SignatureHex, UnixMillis, NodeId, Lane, RiskCoord,
    did::{BostromDid, DidSigner, Signature},
};
use crate::{NodePlacementValidated, QPUShardV1, SafeStepGate, RouteVariant};
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};

/// Final deploy decision outcome.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DeployDecisionOutcome {
    Deploy,
    DeployExperiment,
    Derate(f32),
    Stop,
    Reject,
}

/// A signed deployment decision, ready for anchoring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployDecision {
    pub decision_id: EvidenceHex,
    pub node_placement_hash: EvidenceHex,
    pub decision: DeployDecisionOutcome,
    pub lane: Lane,
    pub ker_k: f32,
    pub ker_e: f32,
    pub ker_r: f32,
    pub vt: f32,
    pub did: BostromDid,
    pub signinghex: SignatureHex,
    pub timestamp: UnixMillis,
    pub evidencehex: EvidenceHex,
}

impl DeployDecision {
    /// Create and sign a new deployment decision.
    pub fn new(
        placement: &NodePlacementValidated<CurrentContract, LaneProd>,
        gate_evaluation: RouteVariant,
        signer: &dyn DidSigner,
    ) -> Result<Self, DecisionError> {
        let decision = match gate_evaluation {
            RouteVariant::Deploy => DeployDecisionOutcome::Deploy,
            RouteVariant::Derate(f) => DeployDecisionOutcome::Derate(f),
            RouteVariant::Stop => DeployDecisionOutcome::Stop,
            RouteVariant::Observe => DeployDecisionOutcome::Reject,
        };

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;

        let mut decision_obj = Self {
            decision_id: EvidenceHex([0; 32]), // placeholder
            node_placement_hash: placement.evidencehex.clone(),
            decision,
            lane: Lane::PROD,
            ker_k: placement.ker_k,
            ker_e: placement.ker_e,
            ker_r: placement.ker_r,
            vt: placement.vt,
            did: signer.did().clone(),
            signinghex: SignatureHex([0; 64]), // placeholder
            timestamp,
            evidencehex: EvidenceHex([0; 32]), // placeholder
        };

        // Compute evidencehex over mutable fields
        decision_obj.evidencehex = decision_obj.compute_evidencehex();

        // Sign the evidencehex
        let signature = signer.sign(&decision_obj.evidencehex.0)?;
        decision_obj.signinghex = SignatureHex(signature.to_bytes());

        // Compute decision_id = hash(evidencehex || signinghex)
        decision_obj.decision_id = decision_obj.compute_decision_id();

        Ok(decision_obj)
    }

    fn compute_evidencehex(&self) -> EvidenceHex {
        let mut hasher = Sha256::new();
        // Canonical order defined in ALN
        hasher.update(&self.node_placement_hash.0);
        hasher.update(&[self.decision as u8]);
        hasher.update(&[self.lane as u8]);
        hasher.update(&self.ker_k.to_le_bytes());
        hasher.update(&self.ker_e.to_le_bytes());
        hasher.update(&self.ker_r.to_le_bytes());
        hasher.update(&self.vt.to_le_bytes());
        hasher.update(self.did.as_bytes());
        hasher.update(&self.timestamp.to_le_bytes());
        let result = hasher.finalize();
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&result);
        EvidenceHex(bytes)
    }

    fn compute_decision_id(&self) -> EvidenceHex {
        let mut hasher = Sha256::new();
        hasher.update(&self.evidencehex.0);
        hasher.update(&self.signinghex.0);
        let result = hasher.finalize();
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&result);
        EvidenceHex(bytes)
    }

    /// Verify the signature against the DID.
    pub fn verify(&self, verifier: &dyn DidVerifier) -> bool {
        if !verifier.verify(&self.evidencehex.0, &self.signinghex.0, &self.did) {
            return false;
        }
        // Also verify decision_id
        self.decision_id == self.compute_decision_id()
    }
}

/// Bostrom DID types (simplified).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BostromDid(String);

impl BostromDid {
    pub fn new(id: &str) -> Self {
        Self(format!("did:bostrom:{}", id))
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

pub trait DidSigner {
    fn did(&self) -> &BostromDid;
    fn sign(&self, message: &[u8]) -> Result<ed25519_dalek::Signature, SignatureError>;
}

pub trait DidVerifier {
    fn verify(&self, message: &[u8], signature: &[u8], did: &BostromDid) -> bool;
}

#[derive(Debug, thiserror::Error)]
pub enum DecisionError {
    #[error("Signature error: {0}")]
    Signature(#[from] SignatureError),
    #[error("Invalid state for decision")]
    InvalidState,
}

#[derive(Debug, thiserror::Error)]
pub struct SignatureError(String);

/// Bostrom anchor record for on-chain proof.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BostromAnchor {
    pub tx_hash: EvidenceHex,
    pub block_height: u64,
    pub decision_hash: EvidenceHex,
    pub validator_set: Vec<BostromDid>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{SigningKey, VerifyingKey};

    struct MockSigner {
        did: BostromDid,
        signing_key: SigningKey,
    }

    impl DidSigner for MockSigner {
        fn did(&self) -> &BostromDid { &self.did }
        fn sign(&self, message: &[u8]) -> Result<ed25519_dalek::Signature, SignatureError> {
            use ed25519_dalek::Signer;
            Ok(self.signing_key.sign(message))
        }
    }

    struct MockVerifier;

    impl DidVerifier for MockVerifier {
        fn verify(&self, message: &[u8], signature: &[u8], did: &BostromDid) -> bool {
            // In real impl, resolve DID to public key
            true // stub
        }
    }

    #[test]
    fn decision_signing_roundtrip() {
        // Test would require a full placement setup
    }
}
