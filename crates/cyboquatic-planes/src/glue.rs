// File: crates/cyboquatic-planes/src/glue.rs

use serde::{Deserialize, Serialize};

use crate::types::{RiskCoord, RiskVector, Residual, LyapunovWeights};
use crate::carbon_plane::{CarbonRaw, CarbonCorridor};
use crate::biodiversity_plane::{BiodiversityRaw, BiodiversityCorridors};
use crate::materials_plane::{MaterialKinetics, MaterialsCorridors};
use crate::hydraulics_plane::{HydraulicsRaw, HydraulicsCorridor};

/// Example plant state metrics required by the eco planes.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeEcoMetrics {
    // Energy & carbon.
    pub mass_processed_kg: f64,
    pub net_sequestered_kg: f64,
    pub energy_kwh: f64,
    pub grid_intensity_kg_per_kwh: f64,

    // Biodiversity.
    pub connectivity_index: f64,
    pub structural_complexity: f64,
    pub colonization_score: f64,

    // Materials.
    pub t90_days: f64,
    pub r_tox: f64,
    pub r_micro: f64,
    pub r_leach_cec: f64,
    pub r_pfas_resid: f64,

    // Hydraulics.
    pub hlr: f64,

    // Other planes (energy & biology) are assumed pre-normalized here.
    pub r_energy: f64,
    pub r_biology: f64,
}

/// Static corridor config for a node (site/technology-specific).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeCorridors {
    pub carbon: CarbonCorridor,
    pub biodiversity: BiodiversityCorridors,
    pub materials: MaterialsCorridors,
    pub hydraulics: HydraulicsCorridor,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodePlaneScores {
    pub rv: RiskVector,
    pub residual: Residual,
    pub carbon_corridor_ok: bool,
    pub biodiversity_corridor_ok: bool,
    pub materials_corridor_ok: bool,
    pub hydraulics_corridor_ok: bool,
}

pub fn compute_planes(
    metrics: &NodeEcoMetrics,
    corridors: &NodeCorridors,
    weights: LyapunovWeights,
) -> NodePlaneScores {
    // Carbon.
    let carbon_raw = CarbonRaw {
        mass_processed_kg: metrics.mass_processed_kg,
        net_sequestered_kg: metrics.net_sequestered_kg,
        energy_kwh: metrics.energy_kwh,
    };
    let carbon_score =
        corridors
            .carbon
            .score(carbon_raw, metrics.grid_intensity_kg_per_kwh);

    // Biodiversity.
    let bio_raw = BiodiversityRaw {
        connectivity_index: metrics.connectivity_index,
        structural_complexity: metrics.structural_complexity,
        colonization_score: metrics.colonization_score,
    };
    let bio_score = corridors.biodiversity.score(bio_raw);

    // Materials.
    let mat_raw = MaterialKinetics {
        t90_days: metrics.t90_days,
        r_tox: metrics.r_tox,
        r_micro: metrics.r_micro,
        r_leach_cec: metrics.r_leach_cec,
        r_pfas_resid: metrics.r_pfas_resid,
    };
    let mat_score = corridors.materials.score(mat_raw);

    // Hydraulics.
    let hyd_raw = HydraulicsRaw { hlr: metrics.hlr };
    let hyd_score = corridors.hydraulics.score(hyd_raw);

    let rv = RiskVector {
        r_energy: RiskCoord::new_clamped(metrics.r_energy),
        r_hydraulics: hyd_score.r_hydraulics,
        r_biology: RiskCoord::new_clamped(metrics.r_biology),
        r_carbon: carbon_score.r_carbon,
        r_materials: mat_score.r_materials,
        r_biodiversity: bio_score.r_biodiversity,
    };

    let residual = rv.residual(weights);

    NodePlaneScores {
        rv,
        residual,
        carbon_corridor_ok: carbon_score.corridor_ok,
        biodiversity_corridor_ok: bio_score.corridor_ok,
        materials_corridor_ok: mat_score.corridor_ok,
        hydraulics_corridor_ok: hyd_score.corridor_ok,
    }
}
