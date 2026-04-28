#include "ecosafety/risk.hpp"
#include <algorithm>
#include <cmath>

namespace ecosafety {

double compute_Vt(const RiskVector& rv,
                  const LyapunovWeights& w) {
    double V = 0.0;
    for (const auto& rc : rv.coords) {
        auto it = std::find_if(
            w.weights.begin(), w.weights.end(),
            [&](const RiskWeight& rw) {
                return std::string(rw.coord_id) == rc.coord_id;
            });
        if (it == w.weights.end()) {
            continue; // no weight => no contribution
        }
        double wi = std::max(0.0, it->weight);
        double ri = std::max(0.0, std::min(1.0, rc.value));
        V += wi * ri * ri;
    }
    return V;
}

bool safestep_ok(double Vt_prev,
                 double Vt_next,
                 const SafeStepConfig& cfg) {
    if (Vt_next > cfg.vt_ceiling) {
        return false;
    }
    // discrete-time Lyapunov: V_{t+1} <= V_t outside small interior
    return Vt_next <= Vt_prev + 1e-9;
}

} // namespace ecosafety
