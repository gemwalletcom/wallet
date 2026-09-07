package com.gemwallet.android.features.asset_select.viewmodels

import com.gemwallet.android.application.assets.cases.GetWalletAssets
import com.gemwallet.android.data.services.gemstone.assets.AssetsSearchService
import com.gemwallet.android.features.asset_select.viewmodels.models.SelectAssetFilters
import com.gemwallet.android.model.AssetBalance
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetInfo
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
import java.math.BigInteger

@OptIn(ExperimentalCoroutinesApi::class)
class SendSelectSearchTest {

    private val fundedAsset = mockAsset(chain = Chain.Solana, name = "Solana", symbol = "SOL")
    private val searchAsset = mockAsset(chain = Chain.Ethereum, name = "USDC", symbol = "USDC")
    private val walletAssetResults = listOf(
        mockAssetInfo(
            asset = fundedAsset,
            balance = AssetBalance.create(fundedAsset, available = BigInteger("1000000000")),
        )
    )
    private val searchResults = listOf(
        mockAssetInfo(
            asset = searchAsset,
            balance = AssetBalance.create(searchAsset, available = BigInteger("1000000")),
        )
    )

    @Test
    fun `empty query uses current wallet assets`() = runTest {
        val searchService = searchService()
        val getWalletAssets = walletAssets()
        val search = SendSelectSearch(searchService, getWalletAssets)
        val filters = MutableStateFlow(
            SelectAssetFilters(
                session = null,
                query = "",
                chainFilter = emptyList(),
                hasBalance = false,
            )
        )

        val result = search.items(filters).first()

        assertEquals(walletAssetResults, result)
        verify(exactly = 1) { getWalletAssets.invoke() }
        verify(exactly = 0) { searchService.search(any(), any(), any(), any()) }
    }

    private fun searchService() = mockk<AssetsSearchService> {
        every { search(any(), any(), any(), any()) } returns flowOf(searchResults)
    }

    private fun walletAssets() = mockk<GetWalletAssets> {
        every { this@mockk() } returns flowOf(walletAssetResults)
    }
}
