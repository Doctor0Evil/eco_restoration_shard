#ifndef QPUDATA_RCALIB_HPP
#define QPUDATA_RCALIB_HPP

#include <cstdint>

namespace qpudata {

struct IngestErrorCounts {
    std::uint32_t n_miss;      // missing mandatory fields
    std::uint32_t n_rfc;       // RFC-4180 violations
    std::uint32_t n_type;      // type mismatches
    std::uint32_t n_schema;    // unknown or invalid schema references
    std::uint32_t n_corridor;  // missing or invalid corridor IDs
    std::uint32_t n_unit;      // inconsistent or unknown units
    std::uint32_t n_varid;     // invalid or unknown varid entries
};

struct RcalibBands {
    double safe_max;  // I_rcalib value at which rcalib == 0
    double gold_max;  // I_rcalib at which rcalib == 0.33
    double hard_max;  // I_rcalib at which rcalib == 1.0
};

struct RcalibWeights {
    double w_miss;
    double w_rfc;
    double w_type;
    double w_schema;
    double w_corridor;
    double w_unit;
    double w_varid;
};

double compute_ingest_error(const IngestErrorCounts& c,
                            const RcalibWeights& w);

double normalize_rcalib(double i_rcalib,
                        const RcalibBands& bands);

} // namespace qpudata

#endif // QPUDATA_RCALIB_HPP
