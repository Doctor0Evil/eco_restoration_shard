#ifndef QPUDATA_RSIGMA_HPP
#define QPUDATA_RSIGMA_HPP

namespace qpudata {

struct SigmaComponents {
    double r_drift;  // sensor drift residual 0..1
    double r_noise;  // noise / precision residual 0..1
    double r_bias;   // systematic bias residual 0..1
    double r_loss;   // data loss / dropout residual 0..1
};

struct SigmaWeights {
    double w_drift;
    double w_noise;
    double w_bias;
    double w_loss;
};

double combine_rsigma(const SigmaComponents& r,
                      const SigmaWeights& w);

} // namespace qpudata

#endif // QPUDATA_RSIGMA_HPP
