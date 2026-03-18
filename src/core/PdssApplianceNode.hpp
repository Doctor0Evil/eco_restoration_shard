#pragma once
#include <string>
#include <cmath>
#include <stdexcept>

namespace bugsappliance {

struct PdssIntent {
    std::string profile;     // e.g. "bedbug_cool_surface_low"
    double intensity01;      // [0,1]
    double duty01;          // [0,1] fraction of cycle ON
    double period_seconds;   // total cycle period
};

// Corridor bands in physical units for each channel
struct ApplianceCorridors {
    double rnoise_safe,   rnoise_gold,   rnoise_hard;
    double rthermal_safe, rthermal_gold, rthermal_hard;
    double rvib_safe,     rvib_gold,     rvib_hard;
    // Lyapunov weights (dimensionless)
    double wnoise;
    double wthermal;
    double wvib;
};

struct ApplianceTelemetry {
    double dBA;           // measured or inferred sound level
    double dT_surface_C;  // surface temperature rise (degC)
    double vib_rms;       // vibration proxy (e.g. mm/s RMS)
};

struct RiskCoords {
    double rnoise;
    double rthermal;
    double rvib;
};

struct NodeResidual {
    RiskCoords r;
    double Vt;
};

inline double normalize_coord(double x,
                              double safe,
                              double gold,
                              double hard)
{
    if (!(safe <= gold && gold <= hard)) {
        throw std::invalid_argument("Invalid corridor bands");
    }
    // Canonical piecewise-linear mapping: safe -> 0, hard -> 1
    if (x <= safe) return 0.0;
    if (x >= hard) return 1.0;
    if (x <= gold) {
        const double span = gold - safe;
        return span > 0.0 ? (x - safe) / span * 0.5 : 0.5;
    }
    const double span = hard - gold;
    return span > 0.0 ? 0.5 + (x - gold) / span * 0.5 : 1.0;
}

inline RiskCoords compute_risk(const ApplianceTelemetry& m,
                               const ApplianceCorridors& c)
{
    RiskCoords r;
    r.rnoise   = normalize_coord(m.dBA,
                                 c.rnoise_safe,
                                 c.rnoise_gold,
                                 c.rnoise_hard);
    r.rthermal = normalize_coord(m.dT_surface_C,
                                 c.rthermal_safe,
                                 c.rthermal_gold,
                                 c.rthermal_hard);
    r.rvib     = normalize_coord(m.vib_rms,
                                 c.rvib_safe,
                                 c.rvib_gold,
                                 c.rvib_hard);
    return r;
}

inline double lyapunov(const RiskCoords& r,
                       const ApplianceCorridors& c)
{
    return c.wnoise   * std::pow(r.rnoise,   2.0)
         + c.wthermal * std::pow(r.rthermal, 2.0)
         + c.wvib     * std::pow(r.rvib,     2.0);
}

enum class SafeStepDecision {
    Ok,
    Derate,
    Stop
};

inline SafeStepDecision safestep(const NodeResidual& prev,
                                 const NodeResidual& next)
{
    // Hard corridor: any r_j >= 1 blocks actuation.
    if (next.r.rnoise >= 1.0 ||
        next.r.rthermal >= 1.0 ||
        next.r.rvib   >= 1.0) {
        return SafeStepDecision::Stop;
    }
    // Lyapunov residual must not increase outside the safe interior.
    // Here we treat any increase as violation; you can add a small
    // epsilon band if you later need numeric tolerance.
    if (next.Vt > prev.Vt) {
        return SafeStepDecision::Derate;
    }
    return SafeStepDecision::Ok;
}

} // namespace bugsappliance
