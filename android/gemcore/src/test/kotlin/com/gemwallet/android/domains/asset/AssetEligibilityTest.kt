package com.gemwallet.android.domains.asset

import com.gemwallet.android.model.AssetFilter
import com.wallet.core.primitives.Chain
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemAssetFilter

class AssetEligibilityTest {

    @Test
    fun `query filters keep the balance distinction and the universe and leave enabled to the query`() {
        assertEquals(
            setOf(AssetFilter.Swappable, AssetFilter.HasAvailableBalance),
            listOf(GemAssetFilter.Enabled, GemAssetFilter.Swappable, GemAssetFilter.HasAvailableBalance).toQueryFilters(),
        )
        assertEquals(setOf(AssetFilter.HasBalance), listOf(GemAssetFilter.Enabled, GemAssetFilter.HasBalance).toQueryFilters())
        assertEquals(
            setOf(AssetFilter.ChainsOrAssetIds(listOf(Chain.Solana), listOf("ethereum"))),
            listOf(GemAssetFilter.ChainsOrAssetIds(listOf("solana"), listOf("ethereum"))).toQueryFilters(),
        )
    }
}
