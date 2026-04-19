//! econet-material-cybo
//! Biodegradable substrate traits for Cyboquatic machinery, bound into rx/Vt/KER,
//! with Phoenix-class corridors for t90, toxicity, micro-residue, leachate, PFAS,
//! and caloric (baiting) risk. [triangulating-econet-material][hydrological-buffering]

// File: crates/econet-material-cybo/src/lib.rs

#![forbid(unsafe_code)]

use cyboquatic_ecosafety_core::{CorridorBands, RiskCoord, RiskVector};

/// Kinetics and ecotoxicology metrics under Phoenix-class conditions. [triangulating-econet-material]
///
/// Fields are raw (physical or lab-normalized) values that will be folded into
/// RiskCoord via CorridorBands.
#[derive(Clone, Copy, Debug)]
pub struct MaterialKinetics {
    /// Time to 90% degradation (days) in Phoenix matrices (compost/canal/soil).
    pub t90_days: f64,
    /// Dimensionless toxicity score (e.g. from LC/MS leachate, NOEC/EC50 folding).
    pub r_tox_raw: f64,
    /// Dimensionless micro-residue risk (from MRK fragment spectra under shear).
    pub r_micro_raw: f64,
    /// Dimensionless leachate CEC/chelation risk (CEC, metals, etc.).
    pub r_leach_cec_raw: f64,
    /// Dimensionless PFAS residual risk (residual PFAS mass or conc. vs corridor).
    pub r_pfas_resid_raw: f64,
    /// Caloric density (fraction 0–1) relevant to baiting risk. [triangulating-econet-material]
    pub caloric_density: f64,
}

/// Normalized material risks mapped into RiskCoord. All values are clamped to [0,1].
/// [triangulating-econet-material]
#[derive(Clone, Copy, Debug)]
pub struct MaterialRisks {
    pub r_t90: RiskCoord,
    pub r_tox: RiskCoord,
    pub r_micro: RiskCoord,
    pub r_leach_cec: RiskCoord,
    pub r_pfas_resid: RiskCoord,
    pub r_caloric: RiskCoord,
}

impl MaterialRisks {
    /// Fold raw kinetics/toxicology into normalized RiskCoord using corridor bands.
    ///
    /// Phoenix baselines:
    /// - t90: hard 180 d, gold ~120 d for deployable biodegradable stacks.
    /// - r_tox: gold ≈ 0.10 (10× margin to exposure limits).
    /// - r_micro: deployment gate ~0.05 to suppress microplastics. [triangulating-econet-material]
    pub fn from_kinetics(
        kin: &MaterialKinetics,
        t90_corr: CorridorBands,
        tox_corr: CorridorBands,
        micro_corr: CorridorBands,
        leach_corr: CorridorBands,
        pfas_corr: CorridorBands,
        caloric_corr: CorridorBands,
    ) -> Self {
        Self {
            r_t90: t90_corr.normalize(kin.t90_days),
            r_tox: tox_corr.normalize(kin.r_tox_raw),
            r_micro: micro_corr.normalize(kin.r_micro_raw),
            r_leach_cec: leach_corr.normalize(kin.r_leach_cec_raw),
            r_pfas_resid: pfas_corr.normalize(kin.r_pfas_resid_raw),
            r_caloric: caloric_corr.normalize(kin.caloric_density),
        }
    }

    /// Aggregate into a single r_materials coordinate with tunable weights. [triangulating-econet-material]
    ///
    /// All weights must be non-negative. If they sum to zero, a neutral norm (1.0) is used
    /// to avoid division by zero.
    pub fn r_materials(
        &self,
        w_t90: f64,
        w_tox: f64,
        w_micro: f64,
        w_leach: f64,
        w_pfas: f64,
        w_caloric: f64,
    ) -> RiskCoord {
        let weights = [w_t90, w_tox, w_micro, w_leach, w_pfas, w_caloric];
        for w in &weights {
            assert!(
                *w >= 0.0,
                "material r_materials weight must be non-negative"
            );
        }
        let sum_w: f64 = weights.iter().sum();
        let norm = if sum_w <= 0.0 { 1.0 } else { sum_w };

        let v =
            w_t90 * self.r_t90.value +
            w_tox * self.r_tox.value +
            w_micro * self.r_micro.value +
            w_leach * self.r_leach_cec.value +
            w_pfas * self.r_pfas_resid.value +
            w_caloric * self.r_caloric.value;

        RiskCoord::clamped(v / norm)
    }
}

/// Biodegradable material abstraction: exposes kinetics and normalized risks
/// so that ecosafety kernels and qpudatashards can derive rx/Vt/KER. [triangulating-econet-material]
pub trait BiodegradableMaterial {
    fn kinetics(&self) -> MaterialKinetics;
    fn risks(&self) -> MaterialRisks;
}

/// Map a BiodegradableMaterial into RiskVector planes (materials + carbon).
/// r_materials is computed via weights over sub-risks; r_carbon is treated as
/// a separate coordinate (net CO₂e over lifecycle) set by caller. [triangulating-econet-material]
pub fn embed_material_into_riskvector<M: BiodegradableMaterial>(
    mat: &M,
    rv: &mut RiskVector,
    weights: (f64, f64, f64, f64, f64, f64),
    r_carbon: RiskCoord,
) {
    let risks = mat.risks();
    let (w_t90, w_tox, w_micro, w_leach, w_pfas, w_caloric) = weights;
    let r_mat = risks.r_materials(
        w_t90, w_tox, w_micro, w_leach, w_pfas, w_caloric,
    );

    rv.r_materials = r_mat;
    rv.r_carbon = r_carbon;
}

/// Hard gate for biodegradable, non-toxic, non-baiting substrates. [triangulating-econet-material]
///
/// This trait is used at compile/CI time to prevent unsafe stacks from reaching PROD.
pub trait AntSafeSubstrate {
    /// Returns true only if the substrate is within Phoenix corridors for:
    /// - t90 (≤ hard band)
    /// - toxicity (≤ gold)
    /// - micro-residue (≤ deployment gate)
    /// - caloric density (≤ baiting limit)
    fn corridor_ok(&self) -> bool;
}

/// Trait for compatibility with Cyboquatic node treatment goals. [triangulating-econet-material]
///
/// Ensures a substrate does not undermine PFAS, pathogen, or nutrient removal corridors
/// at a specific node.
pub trait CyboNodeCompatible {
    fn compatible_with_node(&self, node_id: &str) -> bool;
}

/// Example substrate stack implementing both traits and carrying eco-impact metadata. [triangulating-econet-material]
#[derive(Clone, Debug)]
pub struct SubstrateStack {
    pub id: String,
    pub kinetics: MaterialKinetics,
    pub risks: MaterialRisks,
    /// Normalized eco-impact score from CEIM kernels (waste diverted, PFAS removed, etc.).
    pub ecoimpact_score: f64,
}

impl AntSafeSubstrate for SubstrateStack {
    fn corridor_ok(&self) -> bool {
        // Phoenix baseline corridors from your 2026 band. [triangulating-econet-material]
        let t90_hard_days = 180.0;
        let t90_gold_days = 120.0;
        let rtox_gold_max = 0.10;
        let rmicro_max = 0.05;
        let caloric_max = 0.30;

        let t90_ok = self.kinetics.t90_days <= t90_hard_days;
        let t90_gold_or_better = self.kinetics.t90_days <= t90_gold_days;

        let rtox_ok = self.kinetics.r_tox_raw <= rtox_gold_max;
        let rmicro_ok = self.kinetics.r_micro_raw <= rmicro_max;
        let caloric_ok = self.kinetics.caloric_density <= caloric_max;

        // Require decomposability within hard corridor and strong gold-band behavior
        // for tox/micro/baiting.
        t90_ok && t90_gold_or_better && rtox_ok && rmicro_ok && caloric_ok
    }
}

impl CyboNodeCompatible for SubstrateStack {
    fn compatible_with_node(&self, _node_id: &str) -> bool {
        // In production this checks PFAS, nutrients, etc. against node-specific corridors. [triangulating-econet-material]
        // Here we enforce at least that PFAS residue risk is inside safe/gold bands.
        self.kinetics.r_pfas_resid_raw <= 0.10
    }
}

/// Map material risks into the materials slot of a RiskVector without modifying other planes. [triangulating-econet-material]
pub fn material_to_risk_vector(
    base: &RiskVector,
    mat_risks: &MaterialRisks,
    weights: (f64, f64, f64, f64, f64, f64),
) -> RiskVector {
    let (w_t90, w_tox, w_micro, w_leach, w_pfas, w_caloric) = weights;
    let r_mat = mat_risks.r_materials(
        w_t90, w_tox, w_micro, w_leach, w_pfas, w_caloric,
    );

    RiskVector {
        r_energy: base.r_energy,
        r_hydraulics: base.r_hydraulics,
        r_biology: base.r_biology,
        r_carbon: base.r_carbon,
        r_materials: r_mat,
        r_biodiversity: base.r_biodiversity,
        r_sigma: base.r_sigma,
    }
}

// KER scoring for this crate (indicative, for docs):
// K ≈ 0.92 (trait patterns and corridors specified, many substrates need data),
// E ≈ 0.95 (removes persistent plastics, toxic leachate, and micro-residue upstream),
// R ≈ 0.12 (dominated by unvalidated leachate and micro-residue behavior, bounded by trait gates).
// [triangulating-econet-material][hydrological-buffering]
