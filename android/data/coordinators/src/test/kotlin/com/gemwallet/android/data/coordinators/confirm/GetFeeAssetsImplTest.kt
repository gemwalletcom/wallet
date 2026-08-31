package com.gemwallet.android.data.coordinators.confirm

import com.gemwallet.android.application.session.cases.GetCurrentWalletId
import com.gemwallet.android.data.services.gemstone.stores.GemstoneAssetStore
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.testkit.mockWalletId
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
import com.gemwallet.android.serializer.toJson
import uniffi.gemstone.GemAssetBalance
import uniffi.gemstone.GemFeeAsset
import uniffi.gemstone.GemConfirmServiceInterface

class GetFeeAssetsImplTest {

    private val walletId = mockWalletId()
    private val assetStore = mockk<GemstoneAssetStore>()
    private val getCurrentWalletId = mockk<GetCurrentWalletId> { every { this@mockk() } returns flowOf(walletId) }
    private val confirmService = mockk<GemConfirmServiceInterface>()
    private val subject = GetFeeAssetsImpl(mapOf(Chain.Tempo to ChainFeeAssetProvider(Chain.Tempo, assetStore, getCurrentWalletId, confirmService)))

    @Test
    fun keepsOnlyTheFeeAssetsCoreSelected() = runTest {
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
        every { assetStore.observeAssetsInfoByChain(walletId.id, Chain.Tempo) } returns flowOf(funded.take(1) + unsupported + unfunded)
        every { assetStore.observeHiddenAssetsInfoByChain(walletId.id, Chain.Tempo) } returns flowOf(funded.drop(1))
        every { confirmService.feeAssets(walletId.id, Chain.Tempo.string) } returns funded.map {
            GemFeeAsset(asset = it.asset.toJson(), balance = mockGemAssetBalance(it.asset.id.toIdentifier()), price = null)
        }

        val result = subject(Chain.Tempo).first()

        assertEquals(funded.map { it.asset.id }, result.map { it.asset.id })
    }

    @Test
    fun returnsNoAssetsForAChainWithNoFeeAssets() = runTest {
        assertEquals(0, subject(Chain.Ethereum).first().size)
    }
}

private fun mockGemAssetBalance(assetId: String) = GemAssetBalance(
    assetId = assetId,
    available = "0",
    frozen = "0",
    locked = "0",
    staked = "0",
    pending = "0",
    pendingUnconfirmed = "0",
    rewards = "0",
    reserved = "0",
    withdrawable = "0",
    earn = "0",
    metadata = null,
)
