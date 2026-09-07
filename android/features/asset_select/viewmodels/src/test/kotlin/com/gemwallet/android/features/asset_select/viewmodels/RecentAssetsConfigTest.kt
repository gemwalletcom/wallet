package com.gemwallet.android.features.asset_select.viewmodels

import com.wallet.core.primitives.RecentActivityType
import com.gemwallet.android.application.asset_select.cases.GetRecentAssets
import com.gemwallet.android.application.asset_select.cases.SearchSelectAssets
import com.gemwallet.android.application.asset_select.cases.GetSelectAssetsInfo
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.model.AssetFilter
import com.gemwallet.android.model.RecentAssetsRequest
import io.mockk.mockk
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.GemAssetSelectionServiceInterface

class RecentAssetsConfigTest {

    private val getSession = mockk<GetSession>(relaxed = true)
    private val getRecentAssets = mockk<GetRecentAssets>(relaxed = true)
    private val service = mockk<GemAssetSelectionServiceInterface>(relaxed = true)
    private val searchSelectAssets = mockk<SearchSelectAssets>(relaxed = true)
    private val getSelectAssetsInfo = mockk<GetSelectAssetsInfo>(relaxed = true)

    @Test
    fun `receive shows recents without filters`() {
        val vm = ReceiveSelectViewModel(getSession, searchSelectAssets, getRecentAssets, service)
        assertTrue(vm.flow.recents)
        assertEquals(emptySet<AssetFilter>(), vm.assetFilters())
    }

    @Test
    fun `buy filters recents to buyable`() {
        val vm = BuySelectViewModel(getSession, searchSelectAssets, getRecentAssets, service)
        assertTrue(vm.flow.recents)
        assertEquals(setOf(AssetFilter.Buyable), vm.assetFilters())
    }

    @Test
    fun `send filters recents to has balance`() {
        val vm = SendSelectViewModel(getSession, searchSelectAssets, getSelectAssetsInfo, getRecentAssets, service)
        assertTrue(vm.flow.recents)
        assertEquals(setOf(AssetFilter.HasBalance), vm.assetFilters())
    }

    @Test
    fun `request defaults to all types with no filters`() {
        val request = RecentAssetsRequest()
        assertEquals(com.wallet.core.primitives.RecentActivityType.entries, request.types)
        assertEquals(emptySet<AssetFilter>(), request.filters)
    }

    @Test
    fun `request with filters preserves them`() {
        val request = RecentAssetsRequest(filters = setOf(AssetFilter.Buyable, AssetFilter.HasBalance))
        assertEquals(setOf(AssetFilter.Buyable, AssetFilter.HasBalance), request.filters)
    }
}
