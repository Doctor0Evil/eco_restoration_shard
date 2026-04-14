// SafeStep.kt
// SafeStepGate for Kotlin/Android.

package org.cyboquatic.ecosafety.actuation

import org.cyboquatic.ecosafety.Lane
import org.cyboquatic.ecosafety.RiskCoord
import org.cyboquatic.ecosafety.normalization.CorridorSet
import org.cyboquatic.ecosafety.provenance.QPUShardV1
import kotlin.math.abs

enum class RouteVariant {
    DEPLOY, DERATE, STOP, OBSERVE
}

data class SafeStepConfig(
    val vMax: Float = 0.3f,
    val uMax: Float = 0.4f,
    val vTrendWindow: Int = 10,
    val vTrendThreshold: Float = 0.01f
)

data class ResidualState(
    val vt: Float,
    val ut: Float = 0f
)

interface SafeStepGate {
    fun evaluate(
        residual: ResidualState,
        corridors: CorridorSet,
        recentShards: List<QPUShardV1>,
        lane: Lane
    ): RouteVariant

    fun step(
        residual: ResidualState,
        corridors: CorridorSet,
        recentShards: List<QPUShardV1>,
        lane: Lane,
        requestedAction: Float
    ): Float? {
        return when (evaluate(residual, corridors, recentShards, lane)) {
            RouteVariant.DEPLOY -> requestedAction.coerceIn(0f, 1f)
            RouteVariant.DERATE -> requestedAction * 0.5f
            else -> null
        }
    }
}

class StandardSafeStepGate(
    private val config: SafeStepConfig = SafeStepConfig()
) : SafeStepGate {

    private val vHistory = mutableListOf<Float>()

    override fun evaluate(
        residual: ResidualState,
        corridors: CorridorSet,
        recentShards: List<QPUShardV1>,
        lane: Lane
    ): RouteVariant {
        // 1. Corridor present
        if (!corridors.validate()) return RouteVariant.STOP

        // 2. Residual safe
        if (residual.vt > config.vMax || residual.ut > config.uMax) {
            return if (residual.vt > config.vMax * 1.5f) RouteVariant.STOP else RouteVariant.DERATE
        }

        // 3. KER deployable
        recentShards.lastOrNull()?.let { latest ->
            val kerOk = when (lane) {
                Lane.RESEARCH -> true
                Lane.PILOT -> latest.kerK >= 0.80f && latest.kerE >= 0.75f && latest.kerR <= 0.20f
                Lane.PROD -> latest.kerK >= 0.90f && latest.kerE >= 0.90f && latest.kerR <= 0.13f
            }
            if (!kerOk) return RouteVariant.DERATE
        }

        // 4. Vt non-increase
        if (!checkVtNonIncrease()) return RouteVariant.DERATE

        return RouteVariant.DEPLOY
    }

    override fun step(
        residual: ResidualState,
        corridors: CorridorSet,
        recentShards: List<QPUShardV1>,
        lane: Lane,
        requestedAction: Float
    ): Float? {
        updateHistory(residual.vt)
        return super.step(residual, corridors, recentShards, lane, requestedAction)
    }

    private fun checkVtNonIncrease(): Boolean {
        if (vHistory.size < 2) return true
        val n = minOf(vHistory.size, config.vTrendWindow)
        val recentAvg = vHistory.takeLast(n).average().toFloat()
        val olderAvg = if (vHistory.size > n) {
            vHistory.take(vHistory.size - n).takeLast(n).average().toFloat()
        } else {
            vHistory.first()
        }
        return (recentAvg - olderAvg) <= config.vTrendThreshold
    }

    private fun updateHistory(vt: Float) {
        vHistory.add(vt)
        if (vHistory.size > config.vTrendWindow * 2) {
            vHistory.removeAt(0)
        }
    }
}
