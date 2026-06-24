package com.gemwallet.android.features.asset_select.viewmodels

import com.gemwallet.android.application.asset_select.coordinators.SearchSelectAssets
import com.gemwallet.android.features.asset_select.viewmodels.models.BaseSelectSearch
import com.gemwallet.android.features.asset_select.viewmodels.models.SelectAssetFilters
import com.gemwallet.android.testkit.mockAsset
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

@OptIn(ExperimentalCoroutinesApi::class)
class BaseSelectSearchTest {

    private val ethereum = mockAsset(chain = Chain.Ethereum, name = "Ethereum", symbol = "ETH")
    private val results = listOf(mockAssetInfo(asset = ethereum))

    private fun filters(query: String, limit: Int = 50) = MutableStateFlow(
        SelectAssetFilters(
            session = null,
            query = query,
            chainFilter = emptyList(),
            hasBalance = false,
            tag = null,
            limit = limit,
        )
    )

    @Test
    fun `non-empty query with no matches emits empty list`() = runTest {
        val searchSelectAssets = mockk<SearchSelectAssets> {
            every { this@mockk(any(), any(), any()) } returns flowOf(emptyList())
        }
        val search = BaseSelectSearch(searchSelectAssets)

        val result = search.items(filters("zzqxzzq")).first()

        assertEquals(emptyList<Any>(), result)
    }

    @Test
    fun `query, tags and limit are forwarded to repository search`() = runTest {
        val searchSelectAssets = mockk<SearchSelectAssets> {
            every { this@mockk(any(), any(), any()) } returns flowOf(results)
        }
        val search = BaseSelectSearch(searchSelectAssets)

        val result = search.items(filters(query = "eth", limit = 25)).first()

        assertEquals(results, result)
        verify(exactly = 1) { searchSelectAssets("eth", emptyList(), 25) }
    }
}
