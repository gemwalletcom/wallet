package com.gemwallet.android.features.asset_select.viewmodels

import com.gemwallet.android.application.assets.values.AssetsQueryFilter
import com.gemwallet.android.application.assets.values.AssetsQueryScope
import com.gemwallet.android.data.services.store.queries.AssetsQuery
import com.gemwallet.android.features.asset_select.viewmodels.models.BaseSelectSearch
import com.gemwallet.android.features.asset_select.viewmodels.models.mockSelectAssetFilters
import com.gemwallet.android.model.NO_QUERY_LIMIT
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.WalletId
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

    private val walletId = WalletId("wallet-1")
    private val session = mockSession(wallet = mockWallet(id = walletId))
    private val results = listOf(mockAssetInfo(asset = mockAsset(id = mockAssetId(chain = Chain.Ethereum), name = "Ethereum", symbol = "ETH", decimals = 18)))

    @Test
    fun `non-empty query with no matches emits empty list`() = runTest {
        val assetsQuery = mockk<AssetsQuery> {
            every { this@mockk(any(), any(), any(), any(), any()) } returns flowOf(emptyList())
        }
        val search = BaseSelectSearch(assetsQuery)

        val result = search.items(MutableStateFlow(mockSelectAssetFilters(session = session, query = "zzqxzzq"))).first()

        assertEquals(emptyList<Any>(), result)
    }

    @Test
    fun `query and limit are forwarded to the query for the session wallet`() = runTest {
        val assetsQuery = mockk<AssetsQuery> {
            every { this@mockk(any(), any(), any(), any(), any()) } returns flowOf(results)
        }
        val search = BaseSelectSearch(assetsQuery)

        val result = search.items(MutableStateFlow(mockSelectAssetFilters(session = session, query = "eth", limit = 25))).first()

        assertEquals(results, result)
        verify(exactly = 1) { assetsQuery(walletId, "eth", AssetsQueryScope.Wallet, emptySet(), 25) }
    }

    @Test
    fun `scope and filters come from the flow`() = runTest {
        val assetsQuery = mockk<AssetsQuery> {
            every { this@mockk(any(), any(), any(), any(), any()) } returns flowOf(results)
        }
        val search = BaseSelectSearch(assetsQuery)
        val filters = MutableStateFlow(
            mockSelectAssetFilters(session = session, scope = GemSelectAssetScope.ALL_ASSETS, filters = listOf(GemAssetFilter.Enabled, GemAssetFilter.Buyable)),
        )

        search.items(filters).first()

        verify(exactly = 1) { assetsQuery(walletId, "", AssetsQueryScope.AllAssets, setOf(AssetsQueryFilter.Enabled, AssetsQueryFilter.Buyable), NO_QUERY_LIMIT) }
    }

    @Test
    fun `the chain chips and the balance toggle reach the query as filters`() = runTest {
        val assetsQuery = mockk<AssetsQuery> {
            every { this@mockk(any(), any(), any(), any(), any()) } returns flowOf(results)
        }
        val search = BaseSelectSearch(assetsQuery)
        val filters = MutableStateFlow(
            mockSelectAssetFilters(
                session = session,
                filters = listOf(GemAssetFilter.Buyable, GemAssetFilter.Chains(listOf(Chain.Ethereum.string)), GemAssetFilter.HasBalance),
            ),
        )
        search.items(filters).first()
        verify(exactly = 1) {
            assetsQuery(walletId, "", AssetsQueryScope.Wallet, setOf(AssetsQueryFilter.Buyable, AssetsQueryFilter.Chains(listOf(Chain.Ethereum)), AssetsQueryFilter.HasBalance), NO_QUERY_LIMIT)
        }
    }
}
