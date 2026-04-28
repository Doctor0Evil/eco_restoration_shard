-- File: edge/lua/cyboquatic_node_step.lua

local ffi = require("ffi")

ffi.cdef[[
typedef struct {
  double mass_processed_kg;
  double net_sequestered_kg;
  double energy_kwh;
  double grid_intensity_kg_per_kwh;
  double connectivity_index;
  double structural_complexity;
  double colonization_score;
  double t90_days;
  double r_tox;
  double r_micro;
  double r_leach_cec;
  double r_pfas_resid;
  double hlr;
  double r_energy;
  double r_biology;
} NodeEcoMetrics;

typedef struct {
  double renergy;
  double rhydraulics;
  double rbiology;
  double rcarbon;
  double rmaterials;
  double rbiodiversity;
  double vt;
} NodeRiskVectorOut;

void cyboquatic_compute_risk(const NodeEcoMetrics* metrics,
                             NodeRiskVectorOut* out);
]]

local core = ffi.load("libcyboquatic_planes.so")

local function compute_step(metrics)
  local m = ffi.new("NodeEcoMetrics")
  m.mass_processed_kg = metrics.mass_processed_kg
  m.net_sequestered_kg = metrics.net_sequestered_kg
  m.energy_kwh = metrics.energy_kwh
  m.grid_intensity_kg_per_kwh = metrics.grid_intensity_kg_per_kwh
  m.connectivity_index = metrics.connectivity_index
  m.structural_complexity = metrics.structural_complexity
  m.colonization_score = metrics.colonization_score
  m.t90_days = metrics.t90_days
  m.r_tox = metrics.r_tox
  m.r_micro = metrics.r_micro
  m.r_leach_cec = metrics.r_leach_cec
  m.r_pfas_resid = metrics.r_pfas_resid
  m.hlr = metrics.hlr
  m.r_energy = metrics.r_energy
  m.r_biology = metrics.r_biology

  local out = ffi.new("NodeRiskVectorOut")
  core.cyboquatic_compute_risk(m, out)

  return {
    renergy = out.renergy,
    rhydraulics = out.rhydraulics,
    rbiology = out.rbiology,
    rcarbon = out.rcarbon,
    rmaterials = out.rmaterials,
    rbiodiversity = out.rbiodiversity,
    vt = out.vt,
  }
end

return {
  compute_step = compute_step,
}
