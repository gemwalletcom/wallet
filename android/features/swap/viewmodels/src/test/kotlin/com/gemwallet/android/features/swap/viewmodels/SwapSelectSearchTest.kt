package com.gemwallet.android.features.swap.viewmodels

import com.gemwallet.android.application.swap.coordinators.SearchSwapAssets
import com.gemwallet.android.domains.swap.SwapItemType
import com.gemwallet.android.features.asset_select.viewmodels.models.SelectAssetFilters
import com.gemwallet.android.model.AssetBalance
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAssetHyperCoreUBTC
import com.gemwallet.android.testkit.mockAssetHyperCoreUSDC
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockAssetMetaData
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetTag
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Wallet
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test

class SwapSelectSearchTest {

    private val wallet = mockWallet(accounts = listOf(mockAccount(chain = Chain.HyperCore)))
    private val usdcAsset = mockAssetHyperCoreUSDC()
    private val oppositeAsset = mockAssetHyperCoreUBTC()
    private val swapableMetaData = mockAssetMetaData(isSwapEnabled = true)

    @Test
    fun `pay search forwards opposite receive id and selected type to coordinator`() = runTest {
        val fundedAsset = mockAssetInfo(
            asset = usdcAsset,
            balance = AssetBalance.create(usdcAsset, available = "100000000"),
            walletId = wallet.id,
            metadata = swapableMetaData,
        )
        val captured = mutableListOf<SearchCall>()
        val searchSwapAssets = recordingSearchSwapAssets(captured) { flowOf(listOf(fundedAsset)) }

        val subject = SwapSelectSearch(searchSwapAssets).apply {
            swapItemType.value = SwapItemType.Pay
            receiveId.value = oppositeAsset.id
        }
        val filters = MutableStateFlow(
            SelectAssetFilters(
                session = mockSession(wallet = wallet),
                query = "",
                chainFilter = emptyList(),
                hasBalance = false,
                tag = null,
            )
        )

        val result = subject.items(filters).first()

        assertEquals(listOf(fundedAsset), result)
        val call = captured.last()
        assertEquals(wallet, call.wallet)
        assertEquals(SwapItemType.Pay, call.swapItemType)
        assertEquals(oppositeAsset.id, call.oppositeAssetId)
    }

    @Test
    fun `null swap item type defaults to receive`() = runTest {
        val captured = mutableListOf<SearchCall>()
        val searchSwapAssets = recordingSearchSwapAssets(captured) { flowOf(emptyList()) }

        val subject = SwapSelectSearch(searchSwapAssets)
        val filters = MutableStateFlow(
            SelectAssetFilters(
                session = mockSession(wallet = wallet),
                query = "",
                chainFilter = emptyList(),
                hasBalance = false,
                tag = null,
            )
        )

        subject.items(filters).first()

        assertEquals(SwapItemType.Receive, captured.last().swapItemType)
    }

    private fun recordingSearchSwapAssets(
        sink: MutableList<SearchCall>,
        result: (SearchCall) -> Flow<List<AssetInfo>>,
    ) = object : SearchSwapAssets {
        override fun invoke(
            wallet: Wallet?,
            query: String,
            swapItemType: SwapItemType,
            oppositeAssetId: AssetId?,
            tag: AssetTag?,
        ): Flow<List<AssetInfo>> {
            val call = SearchCall(wallet, query, swapItemType, oppositeAssetId, tag)
            sink += call
            return result(call)
        }
    }

    private data class SearchCall(
        val wallet: Wallet?,
        val query: String,
        val swapItemType: SwapItemType,
        val oppositeAssetId: AssetId?,
        val tag: AssetTag?,
    )
}
