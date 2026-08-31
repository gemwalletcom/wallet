package com.gemwallet.android.features.asset_select.viewmodels

import com.gemwallet.android.application.asset_select.cases.GetRecentAssets
import com.gemwallet.android.application.asset_select.cases.SearchSelectAssets
import com.gemwallet.android.application.asset_select.cases.GetSelectAssetsInfo
import com.gemwallet.android.application.asset_select.cases.SwitchAssetVisibility
import com.gemwallet.android.application.assets.cases.SetAssetPinned
import com.gemwallet.android.application.asset_select.cases.UpdateRecentAsset
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.tokens.cases.SearchTokens
import com.gemwallet.android.model.AssetFilter
import com.gemwallet.android.model.RecentAssetsRequest
import io.mockk.mockk
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.GemAssetConfigService

class RecentAssetsConfigTest {

    private val getSession = mockk<GetSession>(relaxed = true)
    private val getRecentAssets = mockk<GetRecentAssets>(relaxed = true)
    private val updateRecentAsset = mockk<UpdateRecentAsset>(relaxed = true)
    private val switchAssetVisibility = mockk<SwitchAssetVisibility>(relaxed = true)
    private val setAssetPinned = mockk<SetAssetPinned>(relaxed = true)
    private val searchTokensCase = mockk<SearchTokens>(relaxed = true)
    private val searchSelectAssets = mockk<SearchSelectAssets>(relaxed = true)
    private val getSelectAssetsInfo = mockk<GetSelectAssetsInfo>(relaxed = true)

    @Test
    fun `receive shows recents without filters`() {
        val vm = AssetSelectViewModel(
            getSession, searchSelectAssets, getRecentAssets, updateRecentAsset,
            switchAssetVisibility, setAssetPinned, searchTokensCase, GemAssetConfigService(),
        )
        assertTrue(vm.showRecents)
        assertEquals(emptySet<AssetFilter>(), vm.assetFilters())
    }

    @Test
    fun `buy filters recents to buyable`() {
        val vm = BuySelectViewModel(
            getSession, searchSelectAssets, getRecentAssets, updateRecentAsset,
            switchAssetVisibility, setAssetPinned, searchTokensCase, GemAssetConfigService(),
        )
        assertTrue(vm.showRecents)
        assertEquals(setOf(AssetFilter.Buyable), vm.assetFilters())
    }

    @Test
    fun `send filters recents to has balance`() {
        val vm = SendSelectViewModel(
            getSession, searchSelectAssets, getSelectAssetsInfo, getRecentAssets,
            updateRecentAsset, switchAssetVisibility, setAssetPinned, searchTokensCase, GemAssetConfigService(),
        )
        assertTrue(vm.showRecents)
        assertEquals(setOf(AssetFilter.HasBalance), vm.assetFilters())
    }

    @Test
    fun `request defaults to all types with no filters`() {
        val request = RecentAssetsRequest()
        assertEquals(com.gemwallet.android.model.RecentType.entries, request.types)
        assertEquals(emptySet<AssetFilter>(), request.filters)
    }

    @Test
    fun `request with filters preserves them`() {
        val request = RecentAssetsRequest(filters = setOf(AssetFilter.Buyable, AssetFilter.HasBalance))
        assertEquals(setOf(AssetFilter.Buyable, AssetFilter.HasBalance), request.filters)
    }
}
