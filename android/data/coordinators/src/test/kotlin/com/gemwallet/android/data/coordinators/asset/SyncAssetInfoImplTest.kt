package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.coordinators.SyncMissingAssets
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetEthereum
import com.gemwallet.android.testkit.mockAssetFull
import com.gemwallet.android.testkit.mockAssetLink
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.AssetAssociation
import com.wallet.core.primitives.AssetAssociationType
import com.wallet.core.primitives.AssetScore
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.mockk
import kotlinx.coroutines.test.runTest
import org.junit.Test
import uniffi.gemstone.GemAssetsService
import uniffi.gemstone.GemBalanceService
import uniffi.gemstone.GemStreamSubscriptionService

class SyncAssetInfoImplTest {

    private val assetsService = mockk<GemAssetsService>()
    private val balanceService = mockk<GemBalanceService>(relaxed = true)
    private val streamSubscriptionService = mockk<GemStreamSubscriptionService>(relaxed = true)
    private val syncMissingAssets = mockk<SyncMissingAssets>(relaxed = true)
    private val sessionRepository = mockk<SessionRepository>(relaxed = true) {
        coEvery { getCurrentCurrency() } returns Currency.USD
    }

    private val subject = SyncAssetInfoImpl(
        assetsService = assetsService,
        balanceService = balanceService,
        streamSubscriptionService = streamSubscriptionService,
        syncMissingAssets = syncMissingAssets,
        sessionRepository = sessionRepository,
    )

    private val asset = mockAsset()
    private val assetFull = mockAssetFull(
        asset = asset,
        score = AssetScore(rank = 42),
        links = listOf(mockAssetLink()),
    )
    private val wallet = mockWallet(
        id = "wallet-1",
        accounts = listOf(mockAccount(chain = Chain.Bitcoin, address = "bc1-current")),
    )

    @Test
    fun syncAssetInfo_syncsBalanceMetadataAndPricesThroughCore() = runTest {
        coEvery { assetsService.syncAsset("bitcoin", Currency.USD.toJson()) } returns assetFull.toJson()

        subject.syncAssetInfo(asset.id, wallet)

        coVerify { balanceService.update("wallet-1", listOf(asset.id.toIdentifier())) }
        coVerify { assetsService.syncAsset("bitcoin", Currency.USD.toJson()) }
        coVerify { streamSubscriptionService.addPrices(listOf(asset.id.toIdentifier())) }
    }

    @Test
    fun syncAssetInfo_skipsWalletsWithoutAnAccountForTheChain() = runTest {
        val wallet = mockWallet(accounts = listOf(mockAccount(chain = Chain.Ethereum)))

        subject.syncAssetInfo(asset.id, wallet)

        coVerify(exactly = 0) { balanceService.update(any(), any()) }
        coVerify(exactly = 0) { assetsService.syncAsset(any(), any()) }
    }

    @Test
    fun syncAssetInfo_prefetchesAssociatedAssets() = runTest {
        val associatedAssetId = mockAssetEthereum().id
        val assetFull = assetFull.copy(
            associations = listOf(
                AssetAssociation(associatedAssetId, AssetAssociationType.Official),
            ),
        )
        coEvery { assetsService.syncAsset("bitcoin", Currency.USD.toJson()) } returns assetFull.toJson()

        subject.syncAssetInfo(asset.id, wallet)

        coVerify { syncMissingAssets.syncMissingAssets(listOf(associatedAssetId)) }
    }
}
