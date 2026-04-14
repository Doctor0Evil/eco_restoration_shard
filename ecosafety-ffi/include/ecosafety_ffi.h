/**
 * ecosafety_ffi.h
 * C Foreign Function Interface for ecosafety-core.
 * Provides a narrow, formally verified API for embedded controllers.
 * All unsafe operations are gated behind Rust's safety guarantees.
 */

#ifndef ECOSAFETY_FFI_H
#define ECOSAFETY_FFI_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ============================================================================
 * Opaque Handle Types
 * ============================================================================ */

/** Opaque handle to a corridor set (normalization bands). */
typedef struct CorridorSetHandle CorridorSetHandle;

/** Opaque handle to a residual state (Vt, Ut, cached contributions). */
typedef struct ResidualStateHandle ResidualStateHandle;

/** Opaque handle to a SafeStepGate instance. */
typedef struct SafeStepGateHandle SafeStepGateHandle;

/** Opaque handle to a validated node placement. */
typedef struct NodePlacementHandle NodePlacementHandle;

/* ============================================================================
 * Error Codes
 * ============================================================================ */

typedef enum {
    ECOSAFETY_OK = 0,
    ECOSAFETY_ERROR_NULL_PTR = -1,
    ECOSAFETY_ERROR_INVALID_STATE = -2,
    ECOSAFETY_ERROR_OUT_OF_BOUNDS = -3,
    ECOSAFETY_ERROR_CORRIDOR_MISSING = -4,
    ECOSAFETY_ERROR_VT_VIOLATION = -5,
    ECOSAFETY_ERROR_KER_INSUFFICIENT = -6,
} EcosafetyError;

/* ============================================================================
 * CorridorSet API
 * ============================================================================ */

/**
 * Create a default corridor set with canonical 7-coordinate bands.
 * Returns NULL on allocation failure.
 */
CorridorSetHandle* ecosafety_corridorset_default(void);

/**
 * Load corridor set from ALN specification file.
 * Path must be a null-terminated UTF-8 string.
 */
CorridorSetHandle* ecosafety_corridorset_from_aln(const char* path, EcosafetyError* err);

/**
 * Free a corridor set handle.
 */
void ecosafety_corridorset_free(CorridorSetHandle* handle);

/**
 * Normalize a raw measurement to a risk coordinate in [0,1].
 * coord_idx: 0-6 corresponding to canonical order (energy, hydraulic, biology, carbon, materials, dataquality, sigma).
 */
float ecosafety_corridorset_normalize(
    const CorridorSetHandle* handle,
    uint8_t coord_idx,
    float raw_value,
    EcosafetyError* err);

/**
 * Normalize all 7 raw measurements at once (more efficient).
 * raw: array of 7 floats.
 * out: array of 7 normalized risk coordinates.
 */
void ecosafety_corridorset_normalize_all(
    const CorridorSetHandle* handle,
    const float raw[7],
    float out[7],
    EcosafetyError* err);

/**
 * Get weights array (7 floats).
 */
void ecosafety_corridorset_weights(
    const CorridorSetHandle* handle,
    float out[7]);

/* ============================================================================
 * ResidualState API
 * ============================================================================ */

/**
 * Create a new residual state with given initial risk coordinates and weights.
 * r: array of 7 normalized risk coordinates.
 * w: array of 7 weights (should match corridor weights).
 */
ResidualStateHandle* ecosafety_residual_new(
    const float r[7],
    const float w[7],
    EcosafetyError* err);

/**
 * Free residual state handle.
 */
void ecosafety_residual_free(ResidualStateHandle* handle);

/**
 * Get current Vt (Lyapunov residual).
 */
float ecosafety_residual_vt(const ResidualStateHandle* handle);

/**
 * Get current Ut (uncertainty residual, if tracked).
 */
float ecosafety_residual_ut(const ResidualStateHandle* handle);

/**
 * Update a subset of risk coordinates (incremental update).
 * changed_indices: array of coordinate indices to update.
 * new_values: array of new normalized values.
 * count: number of coordinates being updated.
 */
void ecosafety_residual_apply_delta(
    ResidualStateHandle* handle,
    const uint8_t* changed_indices,
    const float* new_values,
    size_t count,
    EcosafetyError* err);

/**
 * Perform full recompute of Vt from current r array.
 */
void ecosafety_residual_recompute(ResidualStateHandle* handle);

/* ============================================================================
 * SafeStepGate API
 * ============================================================================ */

/**
 * Create a standard SafeStepGate with default configuration.
 */
SafeStepGateHandle* ecosafety_safestep_default(void);

/**
 * Create a SafeStepGate with custom configuration.
 */
SafeStepGateHandle* ecosafety_safestep_new(
    float v_max,
    float u_max,
    uint32_t v_trend_window,
    float v_trend_threshold);

/**
 * Free SafeStepGate handle.
 */
void ecosafety_safestep_free(SafeStepGateHandle* handle);

/**
 * Evaluate current state and return a route variant.
 * Returns: 0=DEPLOY, 1=DERATE, 2=STOP, 3=OBSERVE.
 */
uint8_t ecosafety_safestep_evaluate(
    SafeStepGateHandle* gate,
    const ResidualStateHandle* residual,
    const CorridorSetHandle* corridors,
    Lane lane,  /* 0=RESEARCH, 1=PILOT, 2=PROD */
    EcosafetyError* err);

/**
 * Step: evaluate and produce bounded command if safe.
 * requested_action: desired actuator setting in [0,1].
 * Returns: clamped action if safe, or negative error code if blocked.
 */
float ecosafety_safestep_step(
    SafeStepGateHandle* gate,
    ResidualStateHandle* residual,
    const CorridorSetHandle* corridors,
    Lane lane,
    float requested_action,
    EcosafetyError* err);

/* ============================================================================
 * NodePlacement API (Validation)
 * ============================================================================ */

/**
 * Validate raw placement data and produce a validated handle.
 * Returns NULL if validation fails; check err for reason.
 */
NodePlacementHandle* ecosafety_placement_validate(
    const char* node_id,
    const float raw_telemetry[7],
    const CorridorSetHandle* corridors,
    Lane* out_lane,  /* determined lane based on scores */
    EcosafetyError* err);

/**
 * Get KER scores from validated placement.
 */
void ecosafety_placement_ker(
    const NodePlacementHandle* handle,
    float* out_k,
    float* out_e,
    float* out_r);

/**
 * Free placement handle.
 */
void ecosafety_placement_free(NodePlacementHandle* handle);

/* ============================================================================
 * Version Information
 * ============================================================================ */

/** Get the ecosafety-core version string. */
const char* ecosafety_version(void);

#ifdef __cplusplus
}
#endif

#endif /* ECOSAFETY_FFI_H */
