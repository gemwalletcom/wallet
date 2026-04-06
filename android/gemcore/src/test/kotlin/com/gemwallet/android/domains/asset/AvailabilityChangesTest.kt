package com.gemwallet.android.domains.asset

import org.junit.Assert.assertEquals
import org.junit.Test

class AvailabilityChangesTest {

    @Test
    fun calculateAvailabilityChanges_returnsOnlyTrackedChanges() {
        val changes = calculateAvailabilityChanges(
            currentEnabledAssetIds = listOf("bitcoin", "solana"),
            targetEnabledAssetIds = listOf("solana", "ethereum"),
            trackedAssetIds = listOf("bitcoin", "solana"),
        )

        assertEquals(emptyList<String>(), changes.idsToEnable)
        assertEquals(listOf("bitcoin"), changes.idsToDisable)
    }

    @Test
    fun calculateAvailabilityChanges_usesCurrentAndTargetIdsByDefault() {
        val changes = calculateAvailabilityChanges(
            currentEnabledAssetIds = listOf("bitcoin", "ethereum", "ethereum"),
            targetEnabledAssetIds = listOf("ethereum", "solana", "solana"),
        )

        assertEquals(listOf("solana"), changes.idsToEnable)
        assertEquals(listOf("bitcoin"), changes.idsToDisable)
    }
}
