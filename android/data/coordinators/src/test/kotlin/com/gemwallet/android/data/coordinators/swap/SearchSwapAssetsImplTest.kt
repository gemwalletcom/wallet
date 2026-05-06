package com.gemwallet.android.data.coordinators.swap

import com.gemwallet.android.application.swap.coordinators.GetSwapSupported
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.domains.swap.SwapItemType
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.AssetBalance
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAssetHyperCoreHype
import com.gemwallet.android.testkit.mockAssetHyperCoreUBTC
import com.gemwallet.android.testkit.mockAssetHyperCoreUSDC
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockAssetMetaData
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.Chain
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.SwapperAssetList

class SearchSwapAssetsImplTest {

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

        val assetsRepository = mockk<AssetsRepository> {
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

        val subject = SearchSwapAssetsImpl(assetsRepository, getSwapSupported)

        val result = subject.invoke(
            wallet = wallet,
            query = "",
            swapItemType = SwapItemType.Pay,
            oppositeAssetId = oppositeAsset.id,
            tag = null,
        ).first()

        assertEquals(listOf(fundedAsset), result)
    }
}
