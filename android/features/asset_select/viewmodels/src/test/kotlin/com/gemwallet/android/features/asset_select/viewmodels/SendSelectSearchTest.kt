package com.gemwallet.android.features.asset_select.viewmodels

import com.gemwallet.android.application.asset_select.coordinators.GetSelectAssetsInfo
import com.gemwallet.android.application.asset_select.coordinators.SearchSelectAssets
import com.gemwallet.android.features.asset_select.viewmodels.models.SelectAssetFilters
import com.gemwallet.android.model.AssetBalance
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetInfo
import com.wallet.core.primitives.AssetTag
import com.wallet.core.primitives.Chain
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class SendSelectSearchTest {

    private val fundedAsset = mockAsset(chain = Chain.Solana, name = "Solana", symbol = "SOL")
    private val searchAsset = mockAsset(chain = Chain.Ethereum, name = "USDC", symbol = "USDC")
    private val walletAssetResults = listOf(
        mockAssetInfo(
            asset = fundedAsset,
            balance = AssetBalance.create(fundedAsset, available = "1000000000"),
        )
    )
    private val searchResults = listOf(
        mockAssetInfo(
            asset = searchAsset,
            balance = AssetBalance.create(searchAsset, available = "1000000"),
        )
    )

    @Test
    fun `empty query uses current wallet assets`() = runTest {
        val searchSelectAssets = searchCoordinator()
        val getSelectAssetsInfo = assetsInfoCoordinator()
        val search = SendSelectSearch(searchSelectAssets, getSelectAssetsInfo)
        val filters = MutableStateFlow(
            SelectAssetFilters(
                session = null,
                query = "",
                chainFilter = emptyList(),
                hasBalance = false,
                tag = null,
            )
        )

        val result = search.items(filters).first()

        assertEquals(walletAssetResults, result)
        verify(exactly = 1) { getSelectAssetsInfo.invoke() }
        verify(exactly = 0) { searchSelectAssets.invoke(any(), any()) }
    }

    @Test
    fun `selected tag keeps using repository search`() = runTest {
        val searchSelectAssets = searchCoordinator()
        val getSelectAssetsInfo = assetsInfoCoordinator()
        val search = SendSelectSearch(searchSelectAssets, getSelectAssetsInfo)
        val filters = MutableStateFlow(
            SelectAssetFilters(
                session = null,
                query = "",
                chainFilter = emptyList(),
                hasBalance = false,
                tag = AssetTag.Stablecoins,
            )
        )

        val result = search.items(filters).first()

        assertEquals(searchResults, result)
        verify(exactly = 1) { searchSelectAssets.invoke("", listOf(AssetTag.Stablecoins)) }
    }

    private fun searchCoordinator() = mockk<SearchSelectAssets> {
        every { this@mockk(any(), any()) } returns flowOf(searchResults)
    }

    private fun assetsInfoCoordinator() = mockk<GetSelectAssetsInfo> {
        every { this@mockk() } returns flowOf(walletAssetResults)
    }
}
