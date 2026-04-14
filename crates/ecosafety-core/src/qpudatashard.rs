// crates/ecosafety-core/src/qpudatashard.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct QpuShardRow {
    pub nodeid: String,
    pub region: String,
    pub lat: f64,
    pub lon: f64,
    pub medium: String,
    pub nodetype: String,
    pub twindowstart: String,
    pub twindowend: String,

    pub parameter: String,    // e.g. "rPFAS", "rSAT"
    pub rawvalue: f64,
    pub rawunit: String,

    pub rxvalue: f64,
    pub rmin: f64,
    pub rmax: f64,
    pub wi: f64,

    pub vt: f64,
    pub ecoimpactscore: f64,
    pub ecoimpactlevel: String,

    pub speciesid: String,
    pub rspecies: f64,
    pub vspecies: f64,
    pub rsigma: f64,
    pub dt: f64,
    pub kerdeployable: bool,
    pub lane: String,

    pub evidencehex: String,
    pub signinghex: String,
}

impl<const N: usize> From<(&Residual<N>, &str, usize)> for QpuShardRow {
    fn from((residual, nodeid, i): (&Residual<N>, &str, usize)) -> Self {
        // Map the i-th coordinate to a shard row; corridor metadata is injected upstream.
        Self {
            nodeid: nodeid.to_owned(),
            region: "PHX-West-Basin".into(),
            lat: 33.4500,
            lon: -112.1500,
            medium: "canal".into(),
            nodetype: "CANALNODE".into(),
            twindowstart: "2026-04-14T00:00Z".into(),
            twindowend: "2026-04-14T00:05Z".into(),
            parameter: format!("r{}", i),
            rawvalue: f64::NAN,
            rawunit: "".into(),
            rxvalue: residual.r[i],
            rmin: 0.0,
            rmax: 1.0,
            wi: residual.w[i],
            vt: residual.vt,
            ecoimpactscore: 0.0,
            ecoimpactlevel: "Unknown".into(),
            speciesid: "".into(),
            rspecies: 0.0,
            vspecies: 0.0,
            rsigma: 0.0,
            dt: 1.0,
            kerdeployable: false,
            lane: "RESEARCH".into(),
            evidencehex: "0x00".into(),
            signinghex: "0x00".into(),
        }
    }
}
