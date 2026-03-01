use crate::risk::{RiskCoord, Residual};

#[derive(Debug, Clone)]
pub struct NpShearFzSample {
    pub sample_id: String,
    pub cohort_id: String,
    pub subject_age_years: f64,
    pub subject_sex: String,
    pub degeneration_grade: String,
    pub species: String,
    pub region_label: String,
    pub posture_label: String,

    pub hydration_frac_wet: f64,
    pub hydration_sd_wet: f64,
    pub osmotic_pressure_mpa: f64,
    pub pg_content_mg_ml: f64,
    pub collagen_fraction: f64,

    pub test_mode: String,
    pub temp_celsius: f64,
    pub prestrain_lambda: f64,
    pub frequency_min_hz: f64,
    pub frequency_max_hz: f64,
    pub frequency_ref_hz: f64,
    pub method_tag: String,
    pub method_caveat: String,

    pub g0_pa: f64,
    pub ginf_pa: f64,
    pub alpha_fz: f64,
    pub tau1_s: f64,
    pub tau2_s: f64,
    pub fit_r2: f64,
    pub fit_rmse_pa: f64,
    pub validity_band_note: String,

    pub gprime_ref_pa: f64,
    pub gdoubleprime_ref_pa: f64,
    pub tan_delta_ref: f64,

    pub k_design_min_pa: f64,
    pub d_design_min_pas: f64,
    pub e_max_j_per_m3: f64,

    pub r_gprime_soft: f64,
    pub r_gprime_rigid: f64,
    pub r_lossfactor: f64,
    pub r_hydration_low: f64,
    pub r_metadata_gap: f64,

    pub v_disc_residual: f64,

    pub knowledge_factor: f64,
    pub eco_impact_value: f64,
    pub risk_of_harm: f64,

    pub source_paper_id: String,
    pub source_table_figure: String,
    pub notes: String,
}

#[derive(Debug, Clone)]
pub struct NpShearFzCorridors {
    pub region_label: String,
    pub hydration_min_wet: f64,
    pub hydration_max_wet: f64,
    pub gprime_soft_safe_max: f64,
    pub gprime_rigid_safe_min: f64,
    pub tan_delta_min: f64,
    pub tan_delta_max: f64,
    pub e_max_safe_j_per_m3: f64,
    pub w_r_gprime_soft: f64,
    pub w_r_gprime_rigid: f64,
    pub w_r_lossfactor: f64,
    pub w_r_hydration_low: f64,
    pub w_r_metadata_gap: f64,
}

impl NpShearFzSample {
    pub fn corridor_present(&self) -> bool {
        self.hydration_frac_wet >= 0.0
            && self.g0_pa > 0.0
            && self.gprime_ref_pa > 0.0
            && self.frequency_ref_hz > 0.0
            && self.v_disc_residual >= 0.0
    }

    pub fn residual(&self, corr: &NpShearFzCorridors) -> Residual {
        let r_soft = RiskCoord::new(self.r_gprime_soft, corr.w_r_gprime_soft);
        let r_rigid = RiskCoord::new(self.r_gprime_rigid, corr.w_r_gprime_rigid);
        let r_loss = RiskCoord::new(self.r_lossfactor, corr.w_r_lossfactor);
        let r_hydr = RiskCoord::new(self.r_hydration_low, corr.w_r_hydration_low);
        let r_gap = RiskCoord::new(self.r_metadata_gap, corr.w_r_metadata_gap);
        Residual::sum(&[r_soft, r_rigid, r_loss, r_hydr, r_gap])
    }

    pub fn safe_step(prev: &Self, next: &Self, corr: &NpShearFzCorridors) -> bool {
        let ok_hydration = next.hydration_frac_wet >= corr.hydration_min_wet
            && next.hydration_frac_wet <= corr.hydration_max_wet;

        let prev_res = prev.v_disc_residual;
        let next_res = next.v_disc_residual;

        ok_hydration && next_res <= prev_res
    }
}
