// File: sim/cpp/decomposition_mirror.cpp

#include <vector>
#include <cmath>
#include "aln_writer.hpp"
#include "rust_ffi.h"  // generated bindings to materials_plane.rs

struct SubstrateState {
    double mass_kg;
    double temperature_k;
};

void run_decomposition_sim(const SubstrateState& initial,
                           double k_ref,
                           double t_ref_k,
                           double dt_days,
                           int steps) {
    SubstrateState s = initial;
    for (int i = 0; i < steps; ++i) {
        double t_days = i * dt_days;
        double k = k_ref; // or Arrhenius-adjusted if temp varies

        s.mass_kg *= std::exp(-k * dt_days);

        double fraction_remaining = s.mass_kg / initial.mass_kg;
        double t90 = -std::log(0.1) / k;

        MaterialKinetics kin;
        kin.t90_days = t90;
        kin.r_tox = /* from lab panel */;
        kin.r_micro = /* from microresidue tests */;
        kin.r_leach_cec = /* from leachate tests */;
        kin.r_pfas_resid = /* from PFAS panel */;

        MaterialsCorridors corridors = load_material_corridors();
        MaterialsScore score = materials_plane_score(corridors, kin);

        AlnRow row;
        row.set("time_days", t_days);
        row.set("mass_kg", s.mass_kg);
        row.set("t90_days", t90);
        row.set("rmaterials", score.r_materials.value);
        aln_write_row("Decomposition.sim", row);
    }
}
