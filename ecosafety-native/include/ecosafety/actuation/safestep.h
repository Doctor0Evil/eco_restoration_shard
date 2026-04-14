// safestep.h
// SafeStepGate interface and standard implementation for C++.

#ifndef ECOSAFETY_SAFESTEP_H
#define ECOSAFETY_SAFESTEP_H

#include "../risk_vector.h"
#include "../normalization/corridor.h"
#include "../provenance/qpu_shard.h"
#include <vector>
#include <deque>
#include <optional>
#include <cmath>

namespace ecosafety {
namespace actuation {

enum class RouteVariant {
    DEPLOY,
    DERATE,
    STOP,
    OBSERVE
};

struct SafeStepConfig {
    float v_max = 0.3f;
    float u_max = 0.4f;
    size_t v_trend_window = 10;
    float v_trend_threshold = 0.01f;
};

/**
 * SafeStepGate abstract interface.
 */
class SafeStepGate {
public:
    virtual ~SafeStepGate() = default;

    virtual RouteVariant evaluate(
        const ResidualState& residual,
        const CorridorSet& corridors,
        const std::vector<provenance::QPUShardV1>& recent_shards,
        Lane lane) const = 0;

    std::optional<float> step(
        const ResidualState& residual,
        const CorridorSet& corridors,
        const std::vector<provenance::QPUShardV1>& recent_shards,
        Lane lane,
        float requested_action)
    {
        RouteVariant route = evaluate(residual, corridors, recent_shards, lane);
        switch (route) {
            case RouteVariant::DEPLOY:
                return std::clamp(requested_action, 0.0f, 1.0f);
            case RouteVariant::DERATE:
                return requested_action * 0.5f; // default derate
            default:
                return std::nullopt;
        }
    }
};

/**
 * Standard SafeStepGate implementation.
 */
class StandardSafeStepGate : public SafeStepGate {
public:
    explicit StandardSafeStepGate(const SafeStepConfig& cfg = SafeStepConfig{})
        : config_(cfg) {}

    RouteVariant evaluate(
        const ResidualState& residual,
        const CorridorSet& corridors,
        const std::vector<provenance::QPUShardV1>& recent_shards,
        Lane lane) const override
    {
        // 1. Corridor present
        if (!corridors.validate()) return RouteVariant::STOP;

        // 2. Residual safe
        if (residual.vt > config_.v_max || residual.ut > config_.u_max) {
            if (residual.vt > config_.v_max * 1.5f) return RouteVariant::STOP;
            return RouteVariant::DERATE;
        }

        // 3. KER deployable
        if (!recent_shards.empty()) {
            const auto& latest = recent_shards.back();
            bool ker_ok = false;
            switch (lane) {
                case Lane::RESEARCH: ker_ok = true; break;
                case Lane::PILOT: ker_ok = latest.ker_k >= 0.80f && latest.ker_e >= 0.75f && latest.ker_r <= 0.20f; break;
                case Lane::PROD: ker_ok = latest.ker_k >= 0.90f && latest.ker_e >= 0.90f && latest.ker_r <= 0.13f; break;
            }
            if (!ker_ok) return RouteVariant::DERATE;
        }

        // 4. Vt non-increase trend (use internal history)
        if (!check_vt_non_increase()) return RouteVariant::DERATE;

        return RouteVariant::DEPLOY;
    }

    std::optional<float> step(
        const ResidualState& residual,
        const CorridorSet& corridors,
        const std::vector<provenance::QPUShardV1>& recent_shards,
        Lane lane,
        float requested_action)
    {
        update_history(residual.vt);
        return SafeStepGate::step(residual, corridors, recent_shards, lane, requested_action);
    }

private:
    SafeStepConfig config_;
    std::deque<float> v_history_;

    bool check_vt_non_increase() const {
        if (v_history_.size() < 2) return true;
        size_t n = std::min(v_history_.size(), config_.v_trend_window);
        float sum_recent = 0.0f;
        for (size_t i = v_history_.size() - n; i < v_history_.size(); ++i)
            sum_recent += v_history_[i];
        float avg_recent = sum_recent / n;

        float sum_older = 0.0f;
        for (size_t i = 0; i < n && i < v_history_.size() - n; ++i)
            sum_older += v_history_[i];
        float avg_older = (n > 0 && v_history_.size() > n) ? sum_older / n : v_history_.front();

        return (avg_recent - avg_older) <= config_.v_trend_threshold;
    }

    void update_history(float vt) {
        v_history_.push_back(vt);
        if (v_history_.size() > config_.v_trend_window * 2)
            v_history_.pop_front();
    }
};

} // namespace actuation
} // namespace ecosafety

#endif
