package com.gemwallet.android.domains.asset

data class AvailabilityChanges(
    val idsToEnable: List<String>,
    val idsToDisable: List<String>,
)

fun calculateAvailabilityChanges(
    currentEnabledAssetIds: List<String>,
    targetEnabledAssetIds: List<String>,
    trackedAssetIds: List<String> = (currentEnabledAssetIds + targetEnabledAssetIds).distinct(),
): AvailabilityChanges {
    val currentEnabled = currentEnabledAssetIds.toSet()
    val targetEnabled = targetEnabledAssetIds.toSet()
    val tracked = trackedAssetIds.toSet()

    return AvailabilityChanges(
        idsToEnable = (targetEnabled - currentEnabled)
            .filter { it in tracked }
            .sorted(),
        idsToDisable = (currentEnabled - targetEnabled)
            .filter { it in tracked }
            .sorted(),
    )
}
