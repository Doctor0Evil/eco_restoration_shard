//! Node placement validation with compile-time lane enforcement.
//! Uses phantom types to make cross-lane misuse non-representable.

use ecosafety_macros::AlnShard;
use ecosafety_core::{
    EvidenceHex, SignatureHex, UnixMillis, NodeId, RiskCoord,
    Residual, KerDeployable, Lane,
};
use std::marker::PhantomData;

/// ALN contract version tag embedded in type system.
pub struct AlnContractTag<const MAJOR: u16, const MINOR: u16, const PATCH: u16>;
pub type CurrentContract = AlnContractTag<2, 0, 0>;

/// Raw, untrusted node placement data from external source (CSV, API).
#[derive(Debug, Clone)]
pub struct NodePlacementRaw {
    pub node_id: NodeId,
    pub latitude: f64,
    pub longitude: f64,
    pub basin_id: String,
    pub deployment_date: UnixMillis,
    pub raw_telemetry: Vec<f32>, // to be normalized
    // ... other fields
}

/// Validated node placement with contract and lane type parameters.
/// The type parameters C (contract) and L (lane) prevent:
/// - Using placements validated under an old contract with new kernels.
/// - Passing research/pilot placements to production deployment logic.
#[derive(Debug, Clone)]
pub struct NodePlacementValidated<C, L> {
    pub row: NodePlacementRow,
    pub evidencehex: EvidenceHex,
    pub signinghex: Option<SignatureHex>,
    pub ker_k: f32,
    pub ker_e: f32,
    pub ker_r: f32,
    pub vt: f32,
    _contract: PhantomData<C>,
    _lane: PhantomData<L>,
}

/// The validated data row (after normalization and corridor checks).
#[derive(Debug, Clone, AlnShard)]
#[aln_contract(
    family = "NodePlacementV2",
    version = "2.0.0",
    path = "../ecosafety-specs/grammar/NodePlacementGrammar2026v1.aln"
)]
pub struct NodePlacementRow {
    pub node_id: NodeId,
    pub r_energy: RiskCoord,
    pub r_hydraulic: RiskCoord,
    pub r_biology: RiskCoord,
    pub r_carbon: RiskCoord,
    pub r_materials: RiskCoord,
    pub r_dataquality: RiskCoord,
    pub r_sigma: RiskCoord,
    pub timestamp: UnixMillis,
}

impl NodePlacementRaw {
    /// The only way to create a validated placement.
    /// Performs full ALN validation and determines appropriate lane.
    pub fn into_validated(
        self,
        corridors: &CorridorSet,
    ) -> Result<NodePlacementValidated<CurrentContract, LaneProd>, ValidationError> {
        // 1. Normalize raw telemetry using corridor bands
        let normalized = corridors.normalize_all(&self.raw_telemetry)?;

        // 2. Build row and run macro-generated validation
        let row = NodePlacementRow {
            node_id: self.node_id,
            r_energy: normalized[0],
            r_hydraulic: normalized[1],
            r_biology: normalized[2],
            r_carbon: normalized[3],
            r_materials: normalized[4],
            r_dataquality: normalized[5],
            r_sigma: normalized[6],
            timestamp: self.deployment_date,
        };

        // 3. Validate against ALN contract (checks corridors, bounds, completeness)
        row.validate_aln_contract()?;

        // 4. Compute Vt, KER scores
        let vt = row.compute_vt();
        let (ker_k, ker_e, ker_r) = row.compute_ker();

        // 5. Determine lane based on scores and invariants
        let lane = if ker_k >= 0.90 && ker_e >= 0.90 && ker_r <= 0.13 {
            Lane::PROD
        } else if ker_k >= 0.80 && ker_e >= 0.75 && ker_r <= 0.20 {
            Lane::PILOT
        } else {
            Lane::RESEARCH
        };

        // 6. Compute evidencehex from canonical serialization
        let evidencehex = row.compute_evidencehex();

        // 7. Only produce PROD-typed validated placement if lane is PROD
        if lane != Lane::PROD {
            return Err(ValidationError::InsufficientScoreForProd {
                required: (0.90, 0.90, 0.13),
                actual: (ker_k, ker_e, ker_r),
            });
        }

        Ok(NodePlacementValidated {
            row,
            evidencehex,
            signinghex: None,
            ker_k,
            ker_e,
            ker_r,
            vt,
            _contract: PhantomData,
            _lane: PhantomData,
        })
    }
}

/// Deployment decision kernel that ONLY accepts PROD-validated placements.
/// Any attempt to pass a Research or Pilot type will fail at compile time.
pub trait DeployDecisionKernel<C, L> {
    fn decide(&self, placement: &NodePlacementValidated<C, L>) -> DeployDecision;
}

/// Production deployment kernel (only for PROD lane).
pub struct ProdDeployKernel;

impl DeployDecisionKernel<CurrentContract, LaneProd> for ProdDeployKernel {
    fn decide(&self, placement: &NodePlacementValidated<CurrentContract, LaneProd>) -> DeployDecision {
        // Additional runtime checks (e.g., multi-node Vt flow)
        if placement.vt > 0.3 {
            DeployDecision::Derate
        } else {
            DeployDecision::Deploy
        }
    }
}

// Research lane kernel (separate type)
pub struct ResearchDeployKernel;
impl DeployDecisionKernel<CurrentContract, LaneResearch> for ResearchDeployKernel {
    fn decide(&self, placement: &NodePlacementValidated<CurrentContract, LaneResearch>) -> DeployDecision {
        // Allow wider exploration
        DeployDecision::DeployExperiment
    }
}

#[derive(Debug, PartialEq)]
pub enum DeployDecision {
    Deploy,
    DeployExperiment,
    Derate,
    Stop,
}

// Lane type tags
pub struct LaneProd;
pub struct LanePilot;
pub struct LaneResearch;

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Insufficient KER scores for production lane")]
    InsufficientScoreForProd { required: (f32, f32, f32), actual: (f32, f32, f32) },
    #[error("ALN contract violation")]
    ContractViolation(String),
}
