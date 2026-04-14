/**
 * ecosafety_ffi.cpp
 * C++ wrapper implementation that delegates to Rust core via FFI.
 * Provides a safe C++ API for embedded systems.
 */

#include "ecosafety_ffi.h"
#include <memory>
#include <cstring>

// Forward declarations of Rust FFI functions (extern "C")
extern "C" {
    void* ecosafety_rs_corridorset_default(void);
    void* ecosafety_rs_corridorset_from_aln(const char* path);
    void ecosafety_rs_corridorset_free(void* ptr);
    float ecosafety_rs_corridorset_normalize(void* ptr, uint8_t idx, float raw);
    void ecosafety_rs_corridorset_normalize_all(void* ptr, const float* raw, float* out);
    void ecosafety_rs_corridorset_weights(void* ptr, float* out);

    void* ecosafety_rs_residual_new(const float* r, const float* w);
    void ecosafety_rs_residual_free(void* ptr);
    float ecosafety_rs_residual_vt(void* ptr);
    float ecosafety_rs_residual_ut(void* ptr);
    void ecosafety_rs_residual_apply_delta(void* ptr, const uint8_t* idx, const float* vals, size_t cnt);
    void ecosafety_rs_residual_recompute(void* ptr);

    void* ecosafety_rs_safestep_default(void);
    void* ecosafety_rs_safestep_new(float vmax, float umax, uint32_t win, float thresh);
    void ecosafety_rs_safestep_free(void* ptr);
    uint8_t ecosafety_rs_safestep_evaluate(void* gate, void* residual, void* corridors, uint8_t lane);
    float ecosafety_rs_safestep_step(void* gate, void* residual, void* corridors, uint8_t lane, float req);
}

/* ============================================================================
 * CorridorSet Implementation
 * ============================================================================ */

extern "C" CorridorSetHandle* ecosafety_corridorset_default(void) {
    return reinterpret_cast<CorridorSetHandle*>(ecosafety_rs_corridorset_default());
}

extern "C" CorridorSetHandle* ecosafety_corridorset_from_aln(const char* path, EcosafetyError* err) {
    void* ptr = ecosafety_rs_corridorset_from_aln(path);
    if (ptr) {
        if (err) *err = ECOSAFETY_OK;
        return reinterpret_cast<CorridorSetHandle*>(ptr);
    }
    if (err) *err = ECOSAFETY_ERROR_NULL_PTR;
    return nullptr;
}

extern "C" void ecosafety_corridorset_free(CorridorSetHandle* handle) {
    ecosafety_rs_corridorset_free(reinterpret_cast<void*>(handle));
}

extern "C" float ecosafety_corridorset_normalize(
    const CorridorSetHandle* handle,
    uint8_t coord_idx,
    float raw_value,
    EcosafetyError* err)
{
    if (!handle) {
        if (err) *err = ECOSAFETY_ERROR_NULL_PTR;
        return 0.0f;
    }
    if (coord_idx >= 7) {
        if (err) *err = ECOSAFETY_ERROR_OUT_OF_BOUNDS;
        return 0.0f;
    }
    if (err) *err = ECOSAFETY_OK;
    return ecosafety_rs_corridorset_normalize(
        const_cast<void*>(reinterpret_cast<const void*>(handle)),
        coord_idx, raw_value);
}

extern "C" void ecosafety_corridorset_normalize_all(
    const CorridorSetHandle* handle,
    const float raw[7],
    float out[7],
    EcosafetyError* err)
{
    if (!handle) {
        if (err) *err = ECOSAFETY_ERROR_NULL_PTR;
        return;
    }
    ecosafety_rs_corridorset_normalize_all(
        const_cast<void*>(reinterpret_cast<const void*>(handle)), raw, out);
    if (err) *err = ECOSAFETY_OK;
}

extern "C" void ecosafety_corridorset_weights(
    const CorridorSetHandle* handle,
    float out[7])
{
    if (handle) {
        ecosafety_rs_corridorset_weights(
            const_cast<void*>(reinterpret_cast<const void*>(handle)), out);
    }
}

/* ============================================================================
 * ResidualState Implementation
 * ============================================================================ */

extern "C" ResidualStateHandle* ecosafety_residual_new(
    const float r[7],
    const float w[7],
    EcosafetyError* err)
{
    void* ptr = ecosafety_rs_residual_new(r, w);
    if (ptr) {
        if (err) *err = ECOSAFETY_OK;
        return reinterpret_cast<ResidualStateHandle*>(ptr);
    }
    if (err) *err = ECOSAFETY_ERROR_NULL_PTR;
    return nullptr;
}

extern "C" void ecosafety_residual_free(ResidualStateHandle* handle) {
    ecosafety_rs_residual_free(reinterpret_cast<void*>(handle));
}

extern "C" float ecosafety_residual_vt(const ResidualStateHandle* handle) {
    if (!handle) return 0.0f;
    return ecosafety_rs_residual_vt(const_cast<void*>(reinterpret_cast<const void*>(handle)));
}

extern "C" float ecosafety_residual_ut(const ResidualStateHandle* handle) {
    if (!handle) return 0.0f;
    return ecosafety_rs_residual_ut(const_cast<void*>(reinterpret_cast<const void*>(handle)));
}

extern "C" void ecosafety_residual_apply_delta(
    ResidualStateHandle* handle,
    const uint8_t* changed_indices,
    const float* new_values,
    size_t count,
    EcosafetyError* err)
{
    if (!handle) {
        if (err) *err = ECOSAFETY_ERROR_NULL_PTR;
        return;
    }
    ecosafety_rs_residual_apply_delta(reinterpret_cast<void*>(handle), changed_indices, new_values, count);
    if (err) *err = ECOSAFETY_OK;
}

extern "C" void ecosafety_residual_recompute(ResidualStateHandle* handle) {
    if (handle) {
        ecosafety_rs_residual_recompute(reinterpret_cast<void*>(handle));
    }
}

/* ============================================================================
 * SafeStepGate Implementation
 * ============================================================================ */

extern "C" SafeStepGateHandle* ecosafety_safestep_default(void) {
    return reinterpret_cast<SafeStepGateHandle*>(ecosafety_rs_safestep_default());
}

extern "C" SafeStepGateHandle* ecosafety_safestep_new(
    float v_max,
    float u_max,
    uint32_t v_trend_window,
    float v_trend_threshold)
{
    return reinterpret_cast<SafeStepGateHandle*>(
        ecosafety_rs_safestep_new(v_max, u_max, v_trend_window, v_trend_threshold));
}

extern "C" void ecosafety_safestep_free(SafeStepGateHandle* handle) {
    ecosafety_rs_safestep_free(reinterpret_cast<void*>(handle));
}

extern "C" uint8_t ecosafety_safestep_evaluate(
    SafeStepGateHandle* gate,
    const ResidualStateHandle* residual,
    const CorridorSetHandle* corridors,
    Lane lane,
    EcosafetyError* err)
{
    if (!gate || !residual || !corridors) {
        if (err) *err = ECOSAFETY_ERROR_NULL_PTR;
        return 3; // OBSERVE
    }
    if (err) *err = ECOSAFETY_OK;
    return ecosafety_rs_safestep_evaluate(
        reinterpret_cast<void*>(gate),
        const_cast<void*>(reinterpret_cast<const void*>(residual)),
        const_cast<void*>(reinterpret_cast<const void*>(corridors)),
        static_cast<uint8_t>(lane));
}

extern "C" float ecosafety_safestep_step(
    SafeStepGateHandle* gate,
    ResidualStateHandle* residual,
    const CorridorSetHandle* corridors,
    Lane lane,
    float requested_action,
    EcosafetyError* err)
{
    if (!gate || !residual || !corridors) {
        if (err) *err = ECOSAFETY_ERROR_NULL_PTR;
        return -1.0f;
    }
    if (err) *err = ECOSAFETY_OK;
    return ecosafety_rs_safestep_step(
        reinterpret_cast<void*>(gate),
        reinterpret_cast<void*>(residual),
        const_cast<void*>(reinterpret_cast<const void*>(corridors)),
        static_cast<uint8_t>(lane),
        requested_action);
}

/* ============================================================================
 * NodePlacement Implementation (Stub)
 * ============================================================================ */

extern "C" NodePlacementHandle* ecosafety_placement_validate(
    const char* node_id,
    const float raw_telemetry[7],
    const CorridorSetHandle* corridors,
    Lane* out_lane,
    EcosafetyError* err)
{
    // Stub implementation
    if (err) *err = ECOSAFETY_OK;
    return nullptr;
}

extern "C" void ecosafety_placement_ker(
    const NodePlacementHandle* handle,
    float* out_k,
    float* out_e,
    float* out_r)
{
    if (handle && out_k && out_e && out_r) {
        *out_k = 0.95f;
        *out_e = 0.91f;
        *out_r = 0.12f;
    }
}

extern "C" void ecosafety_placement_free(NodePlacementHandle* handle) {
    // Free if needed
}

/* ============================================================================
 * Version
 * ============================================================================ */

extern "C" const char* ecosafety_version(void) {
    return "2.0.0";
}
