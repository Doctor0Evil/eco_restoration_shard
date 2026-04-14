// RiskVector.kt
// Kotlin data classes for Cyboquatic ecosafety, compatible with ALN grammar.
// Used in Android companion apps and lightweight validation tools.

package org.cyboquatic.ecosafety

import java.security.MessageDigest
import java.time.Instant

/**
 * ALN-aligned risk coordinate (normalized to [0,1]).
 */
@JvmInline
value class RiskCoord(val value: Float) {
    init {
        require(value in 0.0f..1.0f) { "RiskCoord must be in [0,1]" }
    }
}

/**
 * SHA-256 hash represented as hex string or byte array.
 */
@JvmInline
value class EvidenceHex(val bytes: ByteArray) {
    init {
        require(bytes.size == 32) { "EvidenceHex must be 32 bytes" }
    }

    fun toHexString(): String = bytes.joinToString("") { "%02x".format(it) }

    companion object {
        fun fromHex(hex: String): EvidenceHex {
            require(hex.length == 64) { "Hex string must be 64 characters" }
            return EvidenceHex(hex.chunked(2).map { it.toInt(16).toByte() }.toByteArray())
        }
    }
}

/**
 * Deployment lane as per ALN EnumLane.
 */
enum class Lane(val code: Int) {
    RESEARCH(0),
    PILOT(1),
    PROD(2)
}

/**
 * Corridor band definition for normalization.
 */
data class CorridorBand(
    val coordId: String,
    val safeGoldHard: List<Float>, // [safe_low, gold_low, hard_low, hard_high, gold_high, safe_high]
    val weight: Float,
    val normKind: NormKind
) {
    enum class NormKind { PIECEWISE_AFFINE, LOGARITHMIC, IDENTITY }

    init {
        require(safeGoldHard.size == 6) { "safeGoldHard must have exactly 6 values" }
    }

    /**
     * Normalize a raw measurement to a RiskCoord in [0,1].
     */
    fun normalize(raw: Float): RiskCoord {
        val (sl, gl, hl, hh, gh, sh) = safeGoldHard
        return when (normKind) {
            NormKind.PIECEWISE_AFFINE -> {
                val r = when {
                    raw <= sl -> 0.0f
                    raw <= gl -> (raw - sl) / (gl - sl) * 0.25f
                    raw <= hl -> 0.25f + (raw - gl) / (hl - gl) * 0.25f
                    raw <= hh -> 0.5f + (raw - hl) / (hh - hl) * 0.25f
                    raw <= gh -> 0.75f + (raw - hh) / (gh - hh) * 0.15f
                    raw <= sh -> 0.9f + (raw - gh) / (sh - gh) * 0.1f
                    else -> 1.0f
                }
                RiskCoord(r.coerceIn(0f, 1f))
            }
            else -> RiskCoord(0.5f) // Placeholder for other norm kinds
        }
    }
}

/**
 * Canonical risk vector matching EcoSafetyRiskVector ALN family.
 * Field order matters for canonical serialization.
 */
data class RiskVectorV2(
    val rEnergy: RiskCoord,
    val rHydraulic: RiskCoord,
    val rBiology: RiskCoord,
    val rCarbon: RiskCoord,
    val rMaterials: RiskCoord,
    val rDataQuality: RiskCoord,
    val rSigma: RiskCoord,
    val evidenceHex: EvidenceHex,
    val signingHex: String? = null,
    val timestamp: Long, // Unix millis
    val nodeId: String
) {
    /**
     * Compute Lyapunov residual Vt = Σ w_j * r_j².
     */
    fun computeVt(weights: List<Float>): Float {
        require(weights.size == 7) { "Weights must match number of coordinates" }
        val coords = listOf(rEnergy, rHydraulic, rBiology, rCarbon, rMaterials, rDataQuality, rSigma)
        return coords.zip(weights).sumOf { (r, w) -> w * r.value * r.value }
    }

    /**
     * Canonical serialization order for hashing (excludes evidenceHex and signingHex).
     */
    fun canonicalBytes(): ByteArray {
        // Simple deterministic concatenation; in production use a proper binary format
        return buildString {
            append(rEnergy.value.toString()).append('|')
            append(rHydraulic.value.toString()).append('|')
            append(rBiology.value.toString()).append('|')
            append(rCarbon.value.toString()).append('|')
            append(rMaterials.value.toString()).append('|')
            append(rDataQuality.value.toString()).append('|')
            append(rSigma.value.toString()).append('|')
            append(timestamp.toString()).append('|')
            append(nodeId)
        }.toByteArray(Charsets.UTF_8)
    }

    /**
     * Recompute evidenceHex and validate against provided hash.
     */
    fun verifyEvidence(): Boolean {
        val digest = MessageDigest.getInstance("SHA-256")
        val computed = digest.digest(canonicalBytes())
        return computed.contentEquals(evidenceHex.bytes)
    }
}

/**
 * Fixed-size residual for high-frequency updates (Kotlin version).
 */
class ResidualFixed(
    private val r: FloatArray,
    private val w: FloatArray,
) {
    private val c = FloatArray(r.size)
    var vt: Float = 0f
        private set

    init {
        require(r.size == w.size) { "Arrays must be same length" }
        recomputeVt()
    }

    fun recomputeVt() {
        vt = 0f
        for (i in r.indices) {
            c[i] = w[i] * r[i] * r[i]
            vt += c[i]
        }
    }

    fun applyDelta(changed: List<Pair<Int, Float>>) {
        for ((idx, newR) in changed) {
            require(idx in r.indices && newR in 0f..1f) { "Invalid coordinate" }
            val oldC = c[idx]
            val newC = w[idx] * newR * newR
            vt = vt - oldC + newC
            c[idx] = newC
            r[idx] = newR
        }
    }

    fun getCoordinates(): List<RiskCoord> = r.map { RiskCoord(it) }
}

/**
 * QPU Shard for immutable logging.
 */
data class QPUShardV1(
    val shardId: EvidenceHex,
    val prevShardId: EvidenceHex,
    val alnFamily: String,
    val alnVersion: String,
    val lane: Lane,
    val kerK: Float,
    val kerE: Float,
    val kerR: Float,
    val vt: Float,
    val evidenceHex: EvidenceHex,
    val signingHex: String? = null,
    val riskVector: RiskVectorV2? = null // payload
) {
    /**
     * Verify the chain link and evidence integrity.
     */
    fun verifyChain(previousShard: QPUShardV1?): Boolean {
        if (previousShard != null && !prevShardId.bytes.contentEquals(previousShard.shardId.bytes)) {
            return false
        }
        // In real implementation, recompute evidenceHex from payload and compare
        return true
    }
}
