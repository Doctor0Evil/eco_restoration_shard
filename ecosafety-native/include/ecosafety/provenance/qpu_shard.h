// qpu_shard.h
// QPU Shard with SHA-256 evidencehex and chain verification.

#ifndef ECOSAFETY_QPU_SHARD_H
#define ECOSAFETY_QPU_SHARD_H

#include "../risk_vector.h"
#include <openssl/sha.h>
#include <string>
#include <sstream>
#include <iomanip>
#include <optional>
#include <variant>

namespace ecosafety {
namespace provenance {

/**
 * Payload variant for EcoSafetyRiskVector family.
 */
struct RiskVectorPayload {
    RiskCoord r_energy;
    RiskCoord r_hydraulic;
    RiskCoord r_biology;
    RiskCoord r_carbon;
    RiskCoord r_materials;
    RiskCoord r_dataquality;
    RiskCoord r_sigma;
    UnixMillis timestamp;
    NodeId node_id;
};

using ShardPayload = std::variant<RiskVectorPayload>; // Extend for other families

/**
 * QPU Shard V1 with cryptographic provenance.
 */
class QPUShardV1 {
public:
    EvidenceHex shard_id;
    EvidenceHex prev_shard_id;
    std::string aln_family;
    std::string aln_version;
    Lane lane;
    float ker_k;
    float ker_e;
    float ker_r;
    float vt;
    EvidenceHex evidencehex;
    std::optional<SignatureHex> signinghex;
    ShardPayload payload;

    /**
     * Compute evidencehex from mutable fields in canonical order.
     */
    EvidenceHex compute_evidencehex() const {
        std::string canonical = canonical_string();
        unsigned char hash[SHA256_DIGEST_LENGTH];
        SHA256(reinterpret_cast<const unsigned char*>(canonical.c_str()),
               canonical.size(), hash);
        EvidenceHex out;
        std::copy(hash, hash + SHA256_DIGEST_LENGTH, out.bytes.begin());
        return out;
    }

    /**
     * Seal the shard: compute evidencehex and shard_id.
     */
    void seal() {
        evidencehex = compute_evidencehex();
        // shard_id = SHA256(prev_shard_id || evidencehex)
        unsigned char hash[SHA256_DIGEST_LENGTH];
        SHA256_CTX ctx;
        SHA256_Init(&ctx);
        SHA256_Update(&ctx, prev_shard_id.bytes.data(), 32);
        SHA256_Update(&ctx, evidencehex.bytes.data(), 32);
        SHA256_Final(hash, &ctx);
        std::copy(hash, hash + SHA256_DIGEST_LENGTH, shard_id.bytes.begin());
    }

    bool verify_evidence() const {
        return evidencehex == compute_evidencehex();
    }

    bool verify_chain(const QPUShardV1* prev) const {
        if (prev && prev_shard_id != prev->shard_id) return false;
        unsigned char hash[SHA256_DIGEST_LENGTH];
        SHA256_CTX ctx;
        SHA256_Init(&ctx);
        SHA256_Update(&ctx, prev_shard_id.bytes.data(), 32);
        SHA256_Update(&ctx, evidencehex.bytes.data(), 32);
        SHA256_Final(hash, &ctx);
        return std::equal(hash, hash + SHA256_DIGEST_LENGTH, shard_id.bytes.begin());
    }

    /**
     * Factory for RiskVector shard.
     */
    static QPUShardV1 new_risk_vector(
        const EvidenceHex& prev_shard_id,
        RiskCoord r_energy, RiskCoord r_hydraulic, RiskCoord r_biology,
        RiskCoord r_carbon, RiskCoord r_materials, RiskCoord r_dataquality,
        RiskCoord r_sigma, UnixMillis timestamp, const NodeId& node_id,
        float ker_k, float ker_e, float ker_r, float vt, Lane lane)
    {
        QPUShardV1 shard;
        shard.prev_shard_id = prev_shard_id;
        shard.aln_family = "EcoSafetyRiskVector";
        shard.aln_version = "2.0.0";
        shard.lane = lane;
        shard.ker_k = ker_k;
        shard.ker_e = ker_e;
        shard.ker_r = ker_r;
        shard.vt = vt;
        shard.payload = RiskVectorPayload{
            r_energy, r_hydraulic, r_biology, r_carbon, r_materials,
            r_dataquality, r_sigma, timestamp, node_id
        };
        shard.seal();
        return shard;
    }

private:
    std::string canonical_string() const {
        std::ostringstream oss;
        oss << std::fixed << std::setprecision(6);
        oss << aln_family << '|' << aln_version << '|' << static_cast<int>(lane) << '|';
        oss << ker_k << '|' << ker_e << '|' << ker_r << '|' << vt << '|';
        if (std::holds_alternative<RiskVectorPayload>(payload)) {
            const auto& p = std::get<RiskVectorPayload>(payload);
            oss << p.r_energy.value << '|' << p.r_hydraulic.value << '|'
                << p.r_biology.value << '|' << p.r_carbon.value << '|'
                << p.r_materials.value << '|' << p.r_dataquality.value << '|'
                << p.r_sigma.value << '|' << p.timestamp << '|' << p.node_id;
        }
        return oss.str();
    }
};

} // namespace provenance
} // namespace ecosafety

#endif
