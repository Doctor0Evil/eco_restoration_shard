// File: android/app/src/main/java/org/cyboquatic/ker/DashboardViewModel.kt

package org.cyboquatic.ker

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch

data class KerSnapshot(
    val nodeId: String,
    val k: Double,
    val e: Double,
    val r: Double,
    val vt: Double,
    val jPerKg: Double?,
)

class DashboardViewModel(
    private val shardRepository: ShardRepository,
) : ViewModel() {

    private val _state = MutableStateFlow<List<KerSnapshot>>(emptyList())
    val state: StateFlow<List<KerSnapshot>> = _state

    fun refresh() {
        viewModelScope.launch {
            val shards = shardRepository.fetchLatestKerShards()
            val snapshots = shards.map { s ->
                KerSnapshot(
                    nodeId = s.nodeId,
                    k = s.k,
                    e = s.e,
                    r = s.r,
                    vt = s.vt,
                    jPerKg = s.jPerKg,
                )
            }
            _state.value = snapshots
        }
    }
}
