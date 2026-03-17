// filename: cyboquatic_c_kernel/src/cyboquatic_kernel.cpp
// destination: cyboquatic_c_kernel/src/cyboquatic_kernel.cpp

#include <cmath>
#include <cstdint>
#include <fstream>
#include <iostream>
#include <string>
#include <vector>

struct Reach {
    double length_m;
    double q_m3s;
    double area_m2;
    double c_in_ngL;
    double c_out_ngL;
    bool has_sat;
};

struct SubstrateState {
    double mass_frac;   // remaining fraction [0,1]
    double k_day;       // first-order decay constant [1/day]
};

struct RiskBands {
    double safe;
    double gold;
    double hard;
};

static inline double clamp01(double x) {
    if (x < 0.0) return 0.0;
    if (x > 1.0) return 1.0;
    return x;
}

static inline double corridor_risk(double x, const RiskBands& b) {
    if (x <= b.safe) return 0.0;
    if (x >= b.hard) return 1.0;
    if (x < b.gold) {
        double t = (x - b.safe) / std::max(b.gold - b.safe, 1e-12);
        return clamp01(0.5 * t);
    } else {
        double t = (x - b.gold) / std::max(b.hard - b.gold, 1e-12);
        return clamp01(0.5 + 0.5 * t);
    }
}

struct SimConfig {
    double dt_s;
    std::uint64_t steps;
};

struct ResidualState {
    double vt;
};

struct KerWindow {
    std::uint64_t total_steps;
    std::uint64_t lyapunov_safe_steps;
    double max_risk;
};

struct ShardWriter {
    std::ofstream out;
    bool header_written;

    ShardWriter(const std::string& path)
        : out(path, std::ios::out), header_written(false) {}

    void write_header() {
        if (header_written) return;
        out << "nodeid,ts,head_m,q_m3s,hlr_m_per_h,c_pfas_ngL,"
               "massloss_frac,r_hlr,r_pfas,r_t90,vt,k,e,r,hexstamp,notes\n";
        header_written = true;
    }

    static std::string escape(const std::string& s) {
        std::string r;
        r.reserve(s.size());
        for (char c : s) {
            if (c == '"') r.push_back('\'');
            else r.push_back(c);
        }
        return r;
    }

    void write_row(const std::string& nodeid,
                   std::uint64_t ts,
                   double head_m,
                   double q_m3s,
                   double hlr_m_per_h,
                   double c_pfas_ngL,
                   double massloss_frac,
                   double r_hlr,
                   double r_pfas,
                   double r_t90,
                   double vt,
                   double k,
                   double e,
                   double r,
                   const std::string& hexstamp,
                   const std::string& notes) {
        write_header();
        out << nodeid << "," << ts << "," << head_m << "," << q_m3s << ","
            << hlr_m_per_h << "," << c_pfas_ngL << "," << massloss_frac << ","
            << r_hlr << "," << r_pfas << "," << r_t90 << "," << vt << ","
            << k << "," << e << "," << r << ","
            << hexstamp << ","
            << "\"" << escape(notes) << "\"\n";
    }
};

int main(int argc, char** argv) {
    if (argc < 3) {
        std::cerr << "usage: cyboquatic_kernel config.txt output.csv\n";
        return 1;
    }

    // In a full implementation, corridors and config are loaded from shards.
    RiskBands hlr_bands{0.0, 0.3, 0.6};
    RiskBands pfas_bands{0.0, 20.0, 70.0};
    RiskBands t90_bands{0.0, 120.0, 180.0};

    SimConfig cfg{60.0, 3600}; // 1 h at 60 s steps

    Reach reach{100.0, 0.29, 4.0, 50.0, 50.0, true};
    SubstrateState sub{1.0, std::log(10.0) / 90.0}; // t90=90d

    ResidualState residual{0.0};
    KerWindow window{0, 0, 0.0};

    ShardWriter writer(argv[2]);

    for (std::uint64_t step = 0; step < cfg.steps; ++step) {
        double dt_day = cfg.dt_s / 86400.0;
        double tau_s = reach.area_m2 * reach.length_m / std::max(reach.q_m3s, 1e-9);
        double lambda_s = (0.01 / 86400.0); // placeholder decay
        double c_old = reach.c_out_ngL;
        double dc_dt = (reach.c_in_ngL - c_old) / std::max(tau_s, 1e-6) - lambda_s * c_old;
        double c_new = c_old + dc_dt * cfg.dt_s;
        if (c_new < 0.0) c_new = 0.0;
        reach.c_out_ngL = c_new;

        sub.mass_frac *= std::exp(-sub.k_day * dt_day);
        if (sub.mass_frac < 0.0) sub.mass_frac = 0.0;

        double depth_m = 1.0;
        double hlr_m_per_h = reach.q_m3s / (reach.area_m2 * depth_m) * 3.6;

        double r_hlr = corridor_risk(hlr_m_per_h, hlr_bands);
        double r_pfas = corridor_risk(reach.c_out_ngL, pfas_bands);
        double t90_est = 90.0;
        double r_t90 = corridor_risk(t90_est, t90_bands);

        double w_hlr = 1.0, w_pfas = 1.0, w_t90 = 1.0;
        double vt_new = w_hlr * r_hlr * r_hlr
                      + w_pfas * r_pfas * r_pfas
                      + w_t90 * r_t90 * r_t90;
        double vt_prev = residual.vt;
        residual.vt = vt_new;

        bool safestep_ok = (residual.vt <= vt_prev + 1e-9);
        window.total_steps += 1;
        if (safestep_ok) window.lyapunov_safe_steps += 1;
        double r_max = std::max(r_t90, std::max(r_hlr, r_pfas));
        if (r_max > window.max_risk) window.max_risk = r_max;

        double k = (window.total_steps == 0)
            ? 0.0
            : static_cast<double>(window.lyapunov_safe_steps)
                / static_cast<double>(window.total_steps);
        double r = window.max_risk;
        double e = clamp01(1.0 - r);

        writer.write_row(
            "PHX-CANAL-01",
            step * static_cast<std::uint64_t>(cfg.dt_s),
            123.5,
            reach.q_m3s,
            hlr_m_per_h,
            reach.c_out_ngL,
            1.0 - sub.mass_frac,
            r_hlr,
            r_pfas,
            r_t90,
            residual.vt,
            k,
            e,
            r,
            "0xa1b2c3d4e5f67890",
            "cyboquatic_c_kernel:v1"
        );
    }

    return 0;
}
