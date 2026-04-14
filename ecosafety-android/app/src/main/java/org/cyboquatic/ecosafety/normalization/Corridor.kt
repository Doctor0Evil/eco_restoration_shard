// Corridor.kt
// Corridor band definitions and normalization for Kotlin/Android.

package org.cyboquatic.ecosafety.normalization

import org.cyboquatic.ecosafety.RiskCoord

/**
 * Six thresholds for piecewise-affine normalization.
 */
data class SafeGoldHard(
    val safeLow: Float,
    val goldLow: Float,
    val hardLow: Float,
    val hardHigh: Float,
    val goldHigh: Float,
    val safeHigh: Float
) {
    init {
        require(safeLow <= goldLow && goldLow <= hardLow &&
                hardLow <= hardHigh && hardHigh <= goldHigh &&
                goldHigh <= safeHigh) {
            "Thresholds must be monotonically increasing"
        }
    }

    fun toList(): List<Float> = listOf(safeLow, goldLow, hardLow, hardHigh, goldHigh, safeHigh)

    companion object {
        fun fromArray(arr: FloatArray): SafeGoldHard {
            require(arr.size == 6)
            return SafeGoldHard(arr[0], arr[1], arr[2], arr[3], arr[4], arr[5])
        }
    }
}

enum class NormKind {
    PIECEWISE_AFFINE, LOGARITHMIC, IDENTITY
}

/**
 * Complete corridor definition for one risk coordinate.
 */
data class CorridorBand(
    val coordId: String,
    val safeGoldHard: SafeGoldHard,
    val weight: Float,
    val normKind: NormKind,
    val unit: String? = null,
    val description: String? = null
) {
    /**
     * Normalize a raw measurement to RiskCoord in [0,1].
     */
    fun normalize(raw: Float): RiskCoord {
        return when (normKind) {
            NormKind.PIECEWISE_AFFINE -> normalizePiecewise(raw)
            NormKind.LOGARITHMIC -> normalizeLog(raw)
            NormKind.IDENTITY -> RiskCoord(raw.coerceIn(0f, 1f))
        }
    }

    private fun normalizePiecewise(raw: Float): RiskCoord {
        val s = safeGoldHard
        val r = when {
            raw <= s.safeLow -> 0f
            raw <= s.goldLow -> (raw - s.safeLow) / (s.goldLow - s.safeLow) * 0.25f
            raw <= s.hardLow -> 0.25f + (raw - s.goldLow) / (s.hardLow - s.goldLow) * 0.25f
            raw <= s.hardHigh -> 0.5f + (raw - s.hardLow) / (s.hardHigh - s.hardLow) * 0.25f
            raw <= s.goldHigh -> 0.75f + (raw - s.hardHigh) / (s.goldHigh - s.hardHigh) * 0.15f
            raw <= s.safeHigh -> 0.9f + (raw - s.goldHigh) / (s.safeHigh - s.goldHigh) * 0.1f
            else -> 1f
        }
        return RiskCoord(r.coerceIn(0f, 1f))
    }

    private fun normalizeLog(raw: Float): RiskCoord {
        // Placeholder
        return RiskCoord(0.5f)
    }
}

/**
 * Set of 7 corridors matching canonical risk vector.
 */
class CorridorSet(
    val bands: List<CorridorBand>
) {
    init {
        require(bands.size == 7) { "Must have exactly 7 corridor bands" }
    }

    fun normalizeAll(raw: List<Float>): List<RiskCoord> {
        require(raw.size == 7)
        return bands.zip(raw).map { (band, r) -> band.normalize(r) }
    }

    fun weights(): List<Float> = bands.map { it.weight }

    fun validate(): Boolean {
        if (bands.any { !it.safeGoldHard.run { safeLow <= goldLow && goldLow <= hardLow && hardLow <= hardHigh && hardHigh <= goldHigh && goldHigh <= safeHigh } }) {
            return false
        }
        val sum = bands.sumOf { it.weight.toDouble() }.toFloat()
        return kotlin.math.abs(sum - 1.0f) < 0.001f
    }

    companion object {
        fun default(): CorridorSet = CorridorSet(
            listOf(
                CorridorBand("r_energy", SafeGoldHard.fromArray(floatArrayOf(0f,0.15f,0.35f,0.55f,0.85f,1f)), 0.12f, NormKind.PIECEWISE_AFFINE, "kWh", "Energy"),
                CorridorBand("r_hydraulic", SafeGoldHard.fromArray(floatArrayOf(0f,0.10f,0.25f,0.40f,0.70f,1f)), 0.18f, NormKind.PIECEWISE_AFFINE, "m³/s", "Hydraulic"),
                CorridorBand("r_biology", SafeGoldHard.fromArray(floatArrayOf(0f,0.08f,0.20f,0.35f,0.60f,1f)), 0.20f, NormKind.PIECEWISE_AFFINE, "mg/L", "Biology"),
                CorridorBand("r_carbon", SafeGoldHard.fromArray(floatArrayOf(0f,0.12f,0.30f,0.50f,0.80f,1f)), 0.15f, NormKind.PIECEWISE_AFFINE, "kg CO₂e", "Carbon"),
                CorridorBand("r_materials", SafeGoldHard.fromArray(floatArrayOf(0f,0.10f,0.25f,0.45f,0.75f,1f)), 0.15f, NormKind.PIECEWISE_AFFINE, "toxicity", "Materials"),
                CorridorBand("r_dataquality", SafeGoldHard.fromArray(floatArrayOf(0f,0.05f,0.15f,0.30f,0.60f,1f)), 0.10f, NormKind.PIECEWISE_AFFINE, "%", "Data quality"),
                CorridorBand("r_sigma", SafeGoldHard.fromArray(floatArrayOf(0f,0.10f,0.25f,0.45f,0.70f,1f)), 0.10f, NormKind.PIECEWISE_AFFINE, "unitless", "Uncertainty")
            )
        )
    }
}
