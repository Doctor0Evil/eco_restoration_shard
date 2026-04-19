// File: native/cyboquatic_longhorizon/src/material_decay_kernel.cpp

#include <cmath>
#include <vector>
#include <string>
#include <fstream>

struct DecayParams {
    double k0_per_day;
    double Ea_kj_mol;
    double temp_C;
};

struct DecayState {
    double t_days;
    double mass_fraction;  // remaining fraction
};

static double arrhenius_k(const DecayParams& p) {
    const double R_kj = 0.008314; // kJ/(mol*K)
    double T = p.temp_C + 273.15;
    return p.k0_per_day * std::exp(-p.Ea_kj_mol / (R_kj * T));
}

static std::vector<DecayState> simulate_decay(
    const DecayParams& p, double t_max_days, double dt_days
) {
    std::vector<DecayState> out;
    out.reserve(static_cast<size_t>(t_max_days / dt_days) + 1);

    double k = arrhenius_k(p);
    double m = 1.0;
    double t = 0.0;

    while (t <= t_max_days) {
        out.push_back(DecayState{t, m});
        // First‑order decay: dm/dt = -k m
        m *= std::exp(-k * dt_days);
        t += dt_days;
    }
    return out;
}

// Compute t90 and write a simple CSV shard.
void run_material_decay_shard(
    const std::string& material_id,
    const DecayParams& p,
    double t_max_days,
    double dt_days,
    const std::string& csv_path
) {
    auto series = simulate_decay(p, t_max_days, dt_days);

    double t90 = t_max_days;
    for (const auto& s : series) {
        if (s.mass_fraction <= 0.10) {
            t90 = s.t_days;
            break;
        }
    }

    std::ofstream out(csv_path, std::ios::out | std::ios::trunc);
    out << "material_id,t90_days,k0_per_day,Ea_kj_mol,temp_C\n";
    out << material_id << ","
        << t90 << ","
        << p.k0_per_day << ","
        << p.Ea_kj_mol << ","
        << p.temp_C << "\n";
    out.close();
}
