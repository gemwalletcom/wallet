package com.gemwallet.android.data.coordinators.confirm

import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.domains.asset.defaultAssets
import com.gemwallet.android.model.AssetBalance
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetInfo
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test

class GetFeeAssetsImplTest {

    private val assetsRepository = mockk<AssetsRepository>()
    private val subject = GetFeeAssetsImpl(mapOf(Chain.Tempo to ChainFeeAssetProvider(Chain.Tempo, assetsRepository)))

    @Test
    fun returnsFundedDefaultTempoAssets() = runTest {
        val supported = Chain.Tempo.defaultAssets
        val funded = supported.dropLast(1).map { asset ->
            mockAssetInfo(asset = asset, balance = AssetBalance.create(asset, available = "1"))
        }
        val unfundedAsset = supported.last()
        val unfunded = mockAssetInfo(
            asset = unfundedAsset,
            balance = AssetBalance.create(unfundedAsset, available = "0"),
        )
        val unsupportedAsset = mockAsset(chain = Chain.Tempo, tokenId = "0x1", type = AssetType.TIP20)
        val unsupported = mockAssetInfo(
            asset = unsupportedAsset,
            balance = AssetBalance.create(unsupportedAsset, available = "1"),
        )
        every { assetsRepository.getAssetsInfoByChain(Chain.Tempo) } returns flowOf(funded.take(1) + unsupported + unfunded)
        every { assetsRepository.getHiddenAssetsInfoByChain(Chain.Tempo) } returns flowOf(funded.drop(1))

        val result = subject(Chain.Tempo).first()

        assertEquals(funded.map { it.asset.id }, result.map { it.asset.id })
    }

    @Test
    fun returnsNoAssetsForUnsupportedChain() = runTest {
        assertEquals(0, subject(Chain.Ethereum).first().size)
    }
}
