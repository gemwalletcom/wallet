package com.gemwallet.android.features.asset_select.viewmodels

import com.wallet.core.primitives.RecentActivityType
import com.gemwallet.android.application.assets.cases.GetWalletAssets
import com.gemwallet.android.data.services.gemstone.assets.AssetsSearchService
import com.gemwallet.android.data.services.gemstone.assets.RecentAssetsService
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
    private val recentAssetsService = mockk<RecentAssetsService>(relaxed = true)
    private val service = mockk<GemAssetSelectionServiceInterface>(relaxed = true)
    private val searchService = mockk<AssetsSearchService>(relaxed = true)
    private val getWalletAssets = mockk<GetWalletAssets>(relaxed = true)

    @Test
    fun `receive shows recents without filters`() {
        val vm = ReceiveSelectViewModel(getSession, searchService, recentAssetsService, service)
        assertTrue(vm.flow.recents)
        assertEquals(emptySet<AssetFilter>(), vm.assetFilters())
    }

    @Test
    fun `buy filters recents to buyable`() {
        val vm = BuySelectViewModel(getSession, searchService, recentAssetsService, service)
        assertTrue(vm.flow.recents)
        assertEquals(setOf(AssetFilter.Buyable), vm.assetFilters())
    }

    @Test
    fun `send filters recents to has balance`() {
        val vm = SendSelectViewModel(getSession, searchService, getWalletAssets, recentAssetsService, service)
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
