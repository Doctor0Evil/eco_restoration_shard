#include "qpudata/rcalib.hpp"
#include <algorithm>

namespace qpudata {

double compute_ingest_error(const IngestErrorCounts& c,
                            const RcalibWeights& w) {
    return  w.w_miss    * static_cast<double>(c.n_miss)
          + w.w_rfc     * static_cast<double>(c.n_rfc)
          + w.w_type    * static_cast<double>(c.n_type)
          + w.w_schema  * static_cast<double>(c.n_schema)
          + w.w_corridor* static_cast<double>(c.n_corridor)
          + w.w_unit    * static_cast<double>(c.n_unit)
          + w.w_varid   * static_cast<double>(c.n_varid);
}

// piecewise-linear 0 -> 0.33 -> 1.0 mapping
double normalize_rcalib(double i_rcalib,
                        const RcalibBands& bands) {
    if (i_rcalib <= 0.0) {
        return 0.0;
    }
    if (i_rcalib <= bands.safe_max) {
        return 0.0;
    }
    if (i_rcalib <= bands.gold_max) {
        double t = (i_rcalib - bands.safe_max)
                 / (bands.gold_max - bands.safe_max);
        return std::max(0.0, std::min(0.33, 0.33 * t));
    }
    if (i_rcalib <= bands.hard_max) {
        double t = (i_rcalib - bands.gold_max)
                 / (bands.hard_max - bands.gold_max);
        return std::max(0.33, std::min(1.0, 0.33 + 0.67 * t));
    }
    return 1.0;
}

} // namespace qpudata
