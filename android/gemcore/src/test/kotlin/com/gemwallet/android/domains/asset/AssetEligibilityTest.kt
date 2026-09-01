package com.gemwallet.android.domains.asset

import com.gemwallet.android.model.AssetFilter
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemAssetAction

class AssetEligibilityTest {

    @Test
    fun `swap pay recents keep the available balance filter distinct from total balance`() {
        assertEquals(
            setOf(AssetFilter.Swappable, AssetFilter.HasAvailableBalance),
            GemAssetAction.SWAP_PAY.recentFilters(),
        )
    }

    @Test
    fun `send recents filter on total balance`() {
        assertEquals(setOf(AssetFilter.HasBalance), GemAssetAction.SEND.recentFilters())
    }
}
