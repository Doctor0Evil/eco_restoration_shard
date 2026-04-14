// corridor.h
// Corridor band definitions and normalization kernels for C++.

#ifndef ECOSAFETY_CORRIDOR_H
#define ECOSAFETY_CORRIDOR_H

#include "../risk_vector.h"
#include <array>
#include <string>
#include <optional>
#include <cmath>
#include <algorithm>

namespace ecosafety {

/**
 * Six thresholds for piecewise-affine normalization.
 */
struct SafeGoldHard {
    float safe_low;
    float gold_low;
    float hard_low;
    float hard_high;
    float gold_high;
    float safe_high;

    static SafeGoldHard from_array(const std::array<float, 6>& arr) {
        return {arr[0], arr[1], arr[2], arr[3], arr[4], arr[5]};
    }

    std::array<float, 6> to_array() const {
        return {safe_low, gold_low, hard_low, hard_high, gold_high, safe_high};
    }

    bool validate() const {
        return safe_low <= gold_low && gold_low <= hard_low &&
               hard_low <= hard_high && hard_high <= gold_high &&
               gold_high <= safe_high;
    }
};

enum class NormKind {
    PIECEWISE_AFFINE,
    LOGARITHMIC,
    IDENTITY
};

/**
 * Complete corridor definition for one risk coordinate.
 */
struct CorridorBand {
    std::string coord_id;
    SafeGoldHard safegoldhard;
    float weight;
    NormKind normkind;
    std::optional<std::string> unit;
    std::optional<std::string> description;

    /**
     * Normalize a raw measurement to a RiskCoord in [0,1].
     */
    RiskCoord normalize(float raw) const {
        switch (normkind) {
            case NormKind::PIECEWISE_AFFINE:
                return normalize_piecewise(raw);
            case NormKind::LOGARITHMIC:
                return normalize_log(raw);
            case NormKind::IDENTITY:
                return RiskCoord(std::clamp(raw, 0.0f, 1.0f));
            default:
                return RiskCoord(0.5f);
        }
    }

private:
    RiskCoord normalize_piecewise(float raw) const {
        const auto& s = safegoldhard;
        float r;
        if (raw <= s.safe_low) {
            r = 0.0f;
        } else if (raw <= s.gold_low) {
            r = (raw - s.safe_low) / (s.gold_low - s.safe_low) * 0.25f;
        } else if (raw <= s.hard_low) {
            r = 0.25f + (raw - s.gold_low) / (s.hard_low - s.gold_low) * 0.25f;
        } else if (raw <= s.hard_high) {
            r = 0.5f + (raw - s.hard_low) / (s.hard_high - s.hard_low) * 0.25f;
        } else if (raw <= s.gold_high) {
            r = 0.75f + (raw - s.hard_high) / (s.gold_high - s.hard_high) * 0.15f;
        } else if (raw <= s.safe_high) {
            r = 0.9f + (raw - s.gold_high) / (s.safe_high - s.gold_high) * 0.1f;
        } else {
            r = 1.0f;
        }
        return RiskCoord(std::clamp(r, 0.0f, 1.0f));
    }

    RiskCoord normalize_log(float /*raw*/) const {
        // Placeholder
        return RiskCoord(0.5f);
    }
};

/**
 * Set of 7 corridors matching the canonical risk vector.
 */
class CorridorSet {
public:
    std::array<CorridorBand, 7> bands;

    CorridorSet() {
        // Default initialization from ALN spec
        bands = {{
            {"r_energy", SafeGoldHard::from_array({0.0f,0.15f,0.35f,0.55f,0.85f,1.0f}), 0.12f, NormKind::PIECEWISE_AFFINE, "kWh", "Energy consumption"},
            {"r_hydraulic", SafeGoldHard::from_array({0.0f,0.10f,0.25f,0.40f,0.70f,1.0f}), 0.18f, NormKind::PIECEWISE_AFFINE, "m³/s", "Hydraulic load"},
            {"r_biology", SafeGoldHard::from_array({0.0f,0.08f,0.20f,0.35f,0.60f,1.0f}), 0.20f, NormKind::PIECEWISE_AFFINE, "mg/L", "Dissolved oxygen"},
            {"r_carbon", SafeGoldHard::from_array({0.0f,0.12f,0.30f,0.50f,0.80f,1.0f}), 0.15f, NormKind::PIECEWISE_AFFINE, "kg CO₂e", "Carbon intensity"},
            {"r_materials", SafeGoldHard::from_array({0.0f,0.10f,0.25f,0.45f,0.75f,1.0f}), 0.15f, NormKind::PIECEWISE_AFFINE, "toxicity", "Material degradation"},
            {"r_dataquality", SafeGoldHard::from_array({0.0f,0.05f,0.15f,0.30f,0.60f,1.0f}), 0.10f, NormKind::PIECEWISE_AFFINE, "%", "Data quality"},
            {"r_sigma", SafeGoldHard::from_array({0.0f,0.10f,0.25f,0.45f,0.70f,1.0f}), 0.10f, NormKind::PIECEWISE_AFFINE, "unitless", "Model uncertainty"}
        }};
    }

    std::array<RiskCoord, 7> normalize_all(const std::array<float, 7>& raw) const {
        std::array<RiskCoord, 7> out;
        for (size_t i = 0; i < 7; ++i) {
            out[i] = bands[i].normalize(raw[i]);
        }
        return out;
    }

    std::array<float, 7> weights() const {
        std::array<float, 7> w;
        for (size_t i = 0; i < 7; ++i) w[i] = bands[i].weight;
        return w;
    }

    bool validate() const {
        for (const auto& b : bands) {
            if (!b.safegoldhard.validate()) return false;
        }
        float sum = 0.0f;
        for (const auto& b : bands) sum += b.weight;
        return std::abs(sum - 1.0f) < 0.001f;
    }
};

} // namespace ecosafety

#endif // ECOSAFETY_CORRIDOR_H
