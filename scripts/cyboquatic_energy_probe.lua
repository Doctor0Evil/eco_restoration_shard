-- File: scripts/cyboquatic_energy_probe.lua

local ffi = require("ffi")

ffi.cdef[[
typedef struct {
  double cin_mg_l;
  double cout_mg_l;
  double flow_m3_s;
  double dt_s;
  double energy_j;
} Sample;

double cyboq_mass_removed_kg(const Sample* samples, int n);
double cyboq_energy_total_j(const Sample* samples, int n);
int    cyboq_specific_energy_j_per_kg(const Sample* samples, int n, double* out);
]]

local lib = ffi.load("libcyboquatic_energy_mass")

local function compute_specific_energy(samples)
  local n = #samples
  local arr = ffi.new("Sample[?]", n)
  for i, s in ipairs(samples) do
    arr[i-1].cin_mg_l   = s.cin_mg_l
    arr[i-1].cout_mg_l  = s.cout_mg_l
    arr[i-1].flow_m3_s  = s.flow_m3_s
    arr[i-1].dt_s       = s.dt_s
    arr[i-1].energy_j   = s.energy_j
  end

  local out = ffi.new("double[1]")
  local ok = lib.cyboq_specific_energy_j_per_kg(arr, n, out)
  if ok == 0 then
    return nil
  end
  return out[0]
end

return {
  compute_specific_energy = compute_specific_energy,
}
