package com.gemwallet.android.domains.asset

import com.gemwallet.android.model.AssetFilter
import com.wallet.core.primitives.Chain
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemAssetAction
import uniffi.gemstone.GemAssetFilter

class AssetEligibilityTest {

    @Test
    fun `query filters keep the balance distinction and the universe and leave enabled to the query`() {
        assertEquals(
            setOf(AssetFilter.Swappable, AssetFilter.HasAvailableBalance),
            GemAssetAction.SWAP_PAY.filters().toQueryFilters(),
        )
        assertEquals(setOf(AssetFilter.HasBalance), GemAssetAction.SEND.filters().toQueryFilters())
        assertEquals(
            setOf(AssetFilter.ChainsOrAssetIds(listOf(Chain.Solana), listOf("ethereum"))),
            listOf(GemAssetFilter.ChainsOrAssetIds(listOf("solana"), listOf("ethereum"))).toQueryFilters(),
        )
    }
}
