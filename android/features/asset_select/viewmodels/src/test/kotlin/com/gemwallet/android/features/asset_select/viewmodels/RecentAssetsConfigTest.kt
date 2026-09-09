package com.gemwallet.android.features.asset_select.viewmodels

import com.gemwallet.android.model.AssetFilter
import com.gemwallet.android.model.RecentAssetsRequest
import com.wallet.core.primitives.RecentActivityType
import org.junit.Assert.assertEquals
import org.junit.Test

class RecentAssetsConfigTest {

    @Test
    fun `request defaults to all types with no filters`() {
        val request = RecentAssetsRequest()
        assertEquals(RecentActivityType.entries, request.types)
        assertEquals(emptySet<AssetFilter>(), request.filters)
    }

    @Test
    fun `request with filters preserves them`() {
        val request = RecentAssetsRequest(filters = setOf(AssetFilter.Buyable, AssetFilter.HasBalance))
        assertEquals(setOf(AssetFilter.Buyable, AssetFilter.HasBalance), request.filters)
    }
}
