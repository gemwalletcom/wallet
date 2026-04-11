package com.gemwallet.android.features.swap.viewmodels

import com.gemwallet.android.cases.swap.GetSwapSupported
import com.gemwallet.android.features.asset_select.viewmodels.models.SelectAssetFilters
import com.gemwallet.android.features.swap.viewmodels.models.SwapItemType
import com.gemwallet.android.model.AssetBalance
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAssetHyperCoreHype
import com.gemwallet.android.testkit.mockAssetHyperCoreUBTC
import com.gemwallet.android.testkit.mockAssetHyperCoreUSDC
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockAssetMetaData
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockWallet
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.Chain
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.SwapperAssetList

class SwapSelectSearchTest {

    private val wallet = mockWallet(accounts = listOf(mockAccount(chain = Chain.HyperCore)))
    private val hypeAsset = mockAssetHyperCoreHype()
    private val usdcAsset = mockAssetHyperCoreUSDC()
    private val oppositeAsset = mockAssetHyperCoreUBTC()
    private val swapableMetaData = mockAssetMetaData(isSwapEnabled = true)

    @Test
    fun `pay search excludes assets without available balance`() = runTest {
        val fundedAsset = mockAssetInfo(
            asset = usdcAsset,
            balance = AssetBalance.create(usdcAsset, available = "100000000"),
            walletId = wallet.id,
            metadata = swapableMetaData,
        )
        val stakedOnlyAsset = mockAssetInfo(
            asset = hypeAsset,
            balance = AssetBalance.create(hypeAsset, available = "0", staked = "500000000"),
            walletId = wallet.id,
            metadata = swapableMetaData,
        )

        val assetsRepository = mockk<com.gemwallet.android.data.repositories.assets.AssetsRepository> {
            every {
                swapSearch(
                    wallet = wallet,
                    query = "",
                    byChains = listOf(Chain.HyperCore),
                    byAssets = listOf(hypeAsset.id, usdcAsset.id),
                    tags = emptyList(),
                )
            } returns flowOf(listOf(stakedOnlyAsset, fundedAsset))
        }
        val getSwapSupported = mockk<GetSwapSupported> {
            every { getSwapSupportChains(oppositeAsset.id) } returns SwapperAssetList(
                chains = listOf(Chain.HyperCore.string),
                assetIds = listOf(hypeAsset.id.toIdentifier(), usdcAsset.id.toIdentifier()),
            )
        }

        val subject = SwapSelectSearch(assetsRepository, getSwapSupported).apply {
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
    }
}
