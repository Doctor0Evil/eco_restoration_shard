// node_placement.h
// C++ type-safe node placement validation using strong enums.
// Lane is part of the type via template parameter (or tag dispatching).

#ifndef ECOSAFETY_NODE_PLACEMENT_H
#define ECOSAFETY_NODE_PLACEMENT_H

#include "../risk_vector.h"
#include <optional>
#include <system_error>

namespace ecosafety {
namespace governance {

// Lane tags for template dispatching
struct LaneProdTag {};
struct LanePilotTag {};
struct LaneResearchTag {};

// Forward declare CorridorSet
struct CorridorSet;

/**
 * Raw placement data before validation.
 */
struct NodePlacementRaw {
    NodeId node_id;
    double latitude;
    double longitude;
    std::string basin_id;
    UnixMillis deployment_date;
    std::vector<float> raw_telemetry;
};

/**
 * Validated placement with lane encoded in type.
 * Template parameter LaneTag prevents cross-lane misuse.
 */
template<typename ContractTag, typename LaneTag>
class NodePlacementValidated {
public:
    // Only friend factory can construct
    static std::optional<NodePlacementValidated> create(
        const NodePlacementRaw& raw,
        const CorridorSet& corridors,
        std::error_code& ec);

    const NodePlacementRow& row() const { return row_; }
    const EvidenceHex& evidencehex() const { return evidencehex_; }
    float ker_k() const { return ker_k_; }
    float ker_e() const { return ker_e_; }
    float ker_r() const { return ker_r_; }
    float vt() const { return vt_; }

private:
    NodePlacementValidated(NodePlacementRow row, EvidenceHex eh,
                           float k, float e, float r, float v)
        : row_(std::move(row)), evidencehex_(eh),
          ker_k_(k), ker_e_(e), ker_r_(r), vt_(v) {}

    NodePlacementRow row_;
    EvidenceHex evidencehex_;
    std::optional<SignatureHex> signinghex_;
    float ker_k_;
    float ker_e_;
    float ker_r_;
    float vt_;
};

// Alias for production-validated placements
using ProdPlacement = NodePlacementValidated<CurrentContract, LaneProdTag>;
using PilotPlacement = NodePlacementValidated<CurrentContract, LanePilotTag>;

/**
 * Deployment decision kernel specialized for production.
 * Accepts only ProdPlacement (compile-time enforced).
 */
class ProdDeployKernel {
public:
    enum class Decision { DEPLOY, DERATE, STOP };

    Decision decide(const ProdPlacement& placement) {
        if (placement.vt() > 0.3f) {
            return Decision::DERATE;
        }
        return Decision::DEPLOY;
    }
};

// Overload for research lane (different signature)
class ResearchDeployKernel {
public:
    enum class Decision { DEPLOY_EXPERIMENT, REJECT };

    Decision decide(const NodePlacementValidated<CurrentContract, LaneResearchTag>& placement) {
        // Allow exploration
        return Decision::DEPLOY_EXPERIMENT;
    }
};

/**
 * Corridor set holding bands for all coordinates.
 */
struct CorridorSet {
    std::array<CorridorBand, 7> bands;

    std::array<RiskCoord, 7> normalize_all(const std::vector<float>& raw) const;
};

} // namespace governance
} // namespace ecosafety

#endif // ECOSAFETY_NODE_PLACEMENT_H
