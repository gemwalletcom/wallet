package com.gemwallet.android.features.asset_select.viewmodels

import com.gemwallet.android.data.services.gemstone.assets.AssetsSearchService
import com.gemwallet.android.features.asset_select.viewmodels.models.BaseSelectSearch
import com.gemwallet.android.features.asset_select.viewmodels.models.mockSelectAssetFilters
import com.gemwallet.android.model.AssetFilter
import com.gemwallet.android.model.NO_QUERY_LIMIT
import com.gemwallet.android.testkit.mockAssetEthereum
import com.gemwallet.android.testkit.mockAssetInfo
import com.wallet.core.primitives.Chain
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemAssetFilter
import uniffi.gemstone.GemSelectAssetScope

@OptIn(ExperimentalCoroutinesApi::class)
class BaseSelectSearchTest {

    private val results = listOf(mockAssetInfo(asset = mockAssetEthereum()))

    @Test
    fun `non-empty query with no matches emits empty list`() = runTest {
        val searchService = mockk<AssetsSearchService> {
            every { search(any(), any(), any(), any()) } returns flowOf(emptyList())
        }
        val search = BaseSelectSearch(searchService)

        val result = search.items(MutableStateFlow(mockSelectAssetFilters(query = "zzqxzzq"))).first()

        assertEquals(emptyList<Any>(), result)
    }

    @Test
    fun `query and limit are forwarded to repository search`() = runTest {
        val searchService = mockk<AssetsSearchService> {
            every { search(any(), any(), any(), any()) } returns flowOf(results)
        }
        val search = BaseSelectSearch(searchService)

        val result = search.items(MutableStateFlow(mockSelectAssetFilters(query = "eth", limit = 25))).first()

        assertEquals(results, result)
        verify(exactly = 1) { searchService.search("eth", false, 25, emptySet()) }
    }

    @Test
    fun `scope and filters come from the flow`() = runTest {
        val searchService = mockk<AssetsSearchService> {
            every { search(any(), any(), any(), any()) } returns flowOf(results)
        }
        val search = BaseSelectSearch(searchService)
        val filters = MutableStateFlow(
            mockSelectAssetFilters(scope = GemSelectAssetScope.ALL_ASSETS, filters = listOf(GemAssetFilter.Enabled, GemAssetFilter.Buyable)),
        )

        search.items(filters).first()

        verify(exactly = 1) { searchService.search("", true, NO_QUERY_LIMIT, setOf(AssetFilter.Enabled, AssetFilter.Buyable)) }
    }

    @Test
    fun `the chain chips and the balance toggle reach the query as filters`() = runTest {
        val searchService = mockk<AssetsSearchService> {
            every { search(any(), any(), any(), any()) } returns flowOf(results)
        }
        val search = BaseSelectSearch(searchService)
        val filters = MutableStateFlow(
            mockSelectAssetFilters(
                filters = listOf(GemAssetFilter.Buyable, GemAssetFilter.Chains(listOf(Chain.Ethereum.string)), GemAssetFilter.HasBalance),
            ),
        )
        search.items(filters).first()
        verify(exactly = 1) {
            searchService.search("", false, NO_QUERY_LIMIT, setOf(AssetFilter.Buyable, AssetFilter.Chains(listOf(Chain.Ethereum)), AssetFilter.HasBalance))
        }
    }
}
