// File: apps/android/src/main/java/org/cyboquatic/ker/DashboardViewModel.kt

data class RiskPoint(
    val nodeId: String,
    val rCarbon: Double,
    val rBiodiversity: Double,
    val rMaterials: Double,
    val vt: Double,
    val kMetric: Double,
    val eMetric: Double,
    val rMetric: Double,
)

class DashboardViewModel(
    private val shardRepository: ShardRepository,
) : ViewModel() {

    val riskPoints: LiveData<List<RiskPoint>> = liveData {
        val shards = shardRepository.loadRecentRiskVectorShards()
        val points = shards.mapNotNull { row ->
            if (!row.verifyHexStamp()) return@mapNotNull null
            RiskPoint(
                nodeId = row["node_id"] ?: return@mapNotNull null,
                rCarbon = row["rcarbon"]!!.toDouble(),
                rBiodiversity = row["rbiodiversity"]!!.toDouble(),
                rMaterials = row["rmaterials"]!!.toDouble(),
                vt = row["vt"]!!.toDouble(),
                kMetric = row["kmetric"]!!.toDouble(),
                eMetric = row["emetric"]!!.toDouble(),
                rMetric = row["rmetric"]!!.toDouble(),
            )
        }
        emit(points)
    }
}
