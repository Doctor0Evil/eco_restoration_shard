// risk_vector.h
// C++ mirror of the ecosafety risk vector and Lyapunov residual.
// Compatible with Rust ecosafety-core via FFI and ALN grammar.

#ifndef ECOSAFETY_RISK_VECTOR_H
#define ECOSAFETY_RISK_VECTOR_H

#include <array>
#include <cstdint>
#include <string>
#include <optional>
#include <cmath>
#include <stdexcept>

namespace ecosafety {

// Type aliases matching ALN coltypes
using RiskCoord = float;          // Normalized to [0,1]
using EvidenceHex = std::array<uint8_t, 32>; // SHA-256
using SignatureHex = std::array<uint8_t, 64>; // Ed25519 signature
using UnixMillis = int64_t;
using NodeId = std::string;       // Bostrom DID fragment

// Deployment lane enum as in ALN
enum class Lane : uint8_t {
    RESEARCH = 0,
    PILOT = 1,
    PROD = 2
};

// Forward declarations
struct CorridorBand;

/**
 * Canonical risk vector matching EcoSafetyRiskVector ALN family.
 * All fields are mutable and must be serialized canonically for evidencehex.
 * Validation must pass corridor and ALN checks before use in production.
 */
struct alignas(32) RiskVectorV2 {
    RiskCoord r_energy;
    RiskCoord r_hydraulic;
    RiskCoord r_biology;
    RiskCoord r_carbon;
    RiskCoord r_materials;
    RiskCoord r_dataquality;
    RiskCoord r_sigma;

    EvidenceHex evidencehex;
    std::optional<SignatureHex> signinghex;

    UnixMillis timestamp;
    NodeId node_id;

    /**
     * Normalize raw measurement using corridor bands.
     * This is the C++ equivalent of the Rust macro-generated normalization.
     */
    static RiskCoord normalize(const CorridorBand& band, float raw_value);

    /**
     * Validate that all risk coordinates are within [0,1] and corridor bands exist.
     * Returns true and sets evidencehex on success.
     */
    bool validate_and_seal(const std::array<CorridorBand, 7>& corridors);

    /**
     * Compute Lyapunov residual Vt = Σ w_j * r_j².
     */
    float compute_vt(const std::array<float, 7>& weights) const;
};

/**
 * Corridor band definition matching ALN's safegoldhard.
 * Six thresholds: [safe_low, gold_low, hard_low, hard_high, gold_high, safe_high]
 */
struct CorridorBand {
    enum class NormKind { PIECEWISE_AFFINE, LOGARITHMIC, IDENTITY };

    std::string coord_id;
    std::array<float, 6> safegoldhard;
    float weight;
    NormKind normkind;

    float normalize(float raw) const;
};

/**
 * Fixed-size residual for high-frequency control (C++ version).
 * Uses stack allocation only, no dynamic memory.
 */
template<size_t N>
class ResidualFixed {
public:
    ResidualFixed(const std::array<float, N>& r_init,
                  const std::array<float, N>& w_init)
        : r_(r_init), w_(w_init)
    {
        recompute_vt();
    }

    void recompute_vt() {
        vt_ = 0.0f;
        for (size_t i = 0; i < N; ++i) {
            c_[i] = w_[i] * r_[i] * r_[i];
            vt_ += c_[i];
        }
    }

    void apply_delta(const std::vector<std::pair<size_t, float>>& changed) {
        for (const auto& [idx, new_r] : changed) {
            if (idx >= N || new_r < 0.0f || new_r > 1.0f) {
                throw std::out_of_range("Risk coordinate out of bounds");
            }
            float old_c = c_[idx];
            float new_c = w_[idx] * new_r * new_r;
            vt_ = vt_ - old_c + new_c;
            c_[idx] = new_c;
            r_[idx] = new_r;
        }
    }

    float vt() const { return vt_; }
    const std::array<float, N>& r() const { return r_; }
    const std::array<float, N>& c() const { return c_; }

private:
    std::array<float, N> r_;
    std::array<float, N> w_;
    std::array<float, N> c_;
    float vt_;
};

// Explicit instantiation for canonical 7-coordinate vector
using Residual7 = ResidualFixed<7>;

/**
 * QPU Shard structure for immutable data logging.
 * All fields except signinghex contribute to evidencehex.
 */
struct QPUShardV1 {
    EvidenceHex shard_id;          // Hash of this shard's content
    EvidenceHex prev_shard_id;     // Chain link
    std::string aln_family;        // e.g., "EcoSafetyRiskVector"
    std::string aln_version;       // Semver
    Lane lane;
    float ker_k;
    float ker_e;
    float ker_r;
    float vt;
    EvidenceHex evidencehex;
    std::optional<SignatureHex> signinghex;

    // Mutable payload (variant based on aln_family)
    RiskVectorV2 risk_vector;      // When aln_family == "EcoSafetyRiskVector"

    /**
     * Canonical serialization for hashing (excludes self hashes).
     * Returns byte vector suitable for SHA-256.
     */
    std::vector<uint8_t> canonical_bytes() const;
};

} // namespace ecosafety

#endif // ECOSAFETY_RISK_VECTOR_H
