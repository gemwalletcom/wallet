package com.gemwallet.android.data.repositories.gemstone

import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.model.AssetBalance
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetEthereum
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockWalletId
import com.wallet.core.primitives.PortfolioAsset
import dagger.Lazy
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Test

class GemstonePortfolioStoreTest {

    @Test
    fun getWalletAssets_returnsOnlyHeldAssets() = runBlocking {
        val bitcoin = mockAsset()
        val ethereum = mockAssetEthereum()
        val walletId = mockWalletId()
        val assetsRepository = mockk<AssetsRepository> {
            every { getAssetsInfo(walletId) } returns flowOf(
                listOf(
                    mockAssetInfo(asset = bitcoin, balance = AssetBalance.create(bitcoin, available = "1000")),
                    mockAssetInfo(asset = ethereum, balance = AssetBalance.create(ethereum)),
                )
            )
        }
        val subject = GemstonePortfolioStore(Lazy { assetsRepository })

        val assets = subject.getWalletAssets(walletId.id).map { it.decodeJson<PortfolioAsset>() }

        assertEquals(listOf(PortfolioAsset(assetId = bitcoin.id, value = "1000")), assets)
    }
}
