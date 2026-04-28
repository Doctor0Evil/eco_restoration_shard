#ifndef ECOSAFETY_RISK_HPP
#define ECOSAFETY_RISK_HPP

#include <vector>

namespace ecosafety {

struct RiskCoord {
    const char* coord_id;   // e.g. "rcarbon", "rmaterials", "rbiodiversity"
    double value;           // normalized 0..1
};

struct RiskWeight {
    const char* coord_id;
    double weight;          // non-negative
};

struct RiskVector {
    std::vector<RiskCoord> coords;
};

struct LyapunovWeights {
    std::vector<RiskWeight> weights;
};

double compute_Vt(const RiskVector& rv,
                  const LyapunovWeights& w);

struct KER {
    double K;   // knowledge factor 0..1
    double E;   // eco-impact 0..1
    double R;   // risk-of-harm 0..1
};

struct SafeStepConfig {
    double vt_ceiling; // e.g. 0.13 or basin-specific
};

bool safestep_ok(double Vt_prev,
                 double Vt_next,
                 const SafeStepConfig& cfg);

} // namespace ecosafety

#endif // ECOSAFETY_RISK_HPP
