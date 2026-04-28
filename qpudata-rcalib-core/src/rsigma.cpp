#include "qpudata/rsigma.hpp"
#include <algorithm>
#include <cmath>

namespace qpudata {

double combine_rsigma(const SigmaComponents& r,
                      const SigmaWeights& w) {
    double v =
        w.w_drift * r.r_drift * r.r_drift +
        w.w_noise * r.r_noise * r.r_noise +
        w.w_bias  * r.r_bias  * r.r_bias  +
        w.w_loss  * r.r_loss  * r.r_loss;

    if (v <= 0.0) {
        return 0.0;
    }
    double rnorm = std::sqrt(v);
    if (rnorm > 1.0) {
        rnorm = 1.0;
    }
    return rnorm;
}

} // namespace qpudata
