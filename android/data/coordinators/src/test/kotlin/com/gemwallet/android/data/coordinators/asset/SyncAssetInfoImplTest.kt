package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.coordinators.PrefetchAssets
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetEthereum
import com.gemwallet.android.testkit.mockAssetFull
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockAssetLink
import com.gemwallet.android.testkit.mockWallet
import com.gemwallet.android.testkit.mockWalletId
import com.wallet.core.primitives.AssetAssociation
import com.wallet.core.primitives.AssetAssociationType
import com.wallet.core.primitives.AssetMetaData
import com.wallet.core.primitives.AssetScore
import com.wallet.core.primitives.Chain
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import org.junit.Test
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.wallet.core.primitives.Currency
import uniffi.gemstone.GemAssetsService
import uniffi.gemstone.GemStreamSubscriptionService
import com.gemwallet.android.ext.toIdentifier

class SyncAssetInfoImplTest {

    private val assetsService = mockk<GemAssetsService>()
    private val assetsRepository = mockk<AssetsRepository>(relaxed = true)
    private val streamSubscriptionService = mockk<GemStreamSubscriptionService>(relaxed = true)
    private val prefetchAssets = mockk<PrefetchAssets>(relaxed = true)
    private val sessionRepository = mockk<SessionRepository>(relaxed = true) {
        coEvery { getCurrentCurrency() } returns Currency.USD
    }

    private val subject = SyncAssetInfoImpl(
        assetsService = assetsService,
        assetsRepository = assetsRepository,
        streamSubscriptionService = streamSubscriptionService,
        prefetchAssets = prefetchAssets,
        sessionRepository = sessionRepository,
    )

    private val asset = mockAsset()
    private val assetMetadata = AssetMetaData(
        isEnabled = true,
        isBalanceEnabled = true,
        isBuyEnabled = true,
        isSellEnabled = true,
        isSwapEnabled = true,
        isStakeEnabled = false,
        isEarnEnabled = false,
        isPinned = false,
        isActive = true,
        rankScore = 42,
    )

    private val assetFull = mockAssetFull(
        asset = asset,
        score = AssetScore(rank = 42),
        links = listOf(mockAssetLink()),
    )

    @Test
    fun syncAssetInfo_addsCurrentWalletAssetWhenOnlyForeignWalletAssetExists() = runTest {
        val wallet = mockWallet(
            id = "wallet-1",
            accounts = listOf(mockAccount(chain = Chain.Bitcoin, address = "bc1-current")),
        )
        val foreignWalletAsset = mockAssetInfo(
            asset = asset,
            walletId = mockWalletId("wallet-2"),
            owner = mockAccount(chain = Chain.Bitcoin, address = "bc1-foreign"),
        ).copy(metadata = assetMetadata)

        every { assetsRepository.getAssetInfo(asset.id) } returns flowOf(null)
        every { assetsRepository.getTokenInfo(asset.id) } returns flowOf(foreignWalletAsset)
        coEvery { assetsService.syncAsset("bitcoin", Currency.USD.toJson()) } returns assetFull.toJson()

        subject.syncAssetInfo(asset.id, wallet)

        coVerify {
            assetsRepository.linkAssetToWallet(
                walletId = "wallet-1",
                assetId = asset.id,
                visible = true,
            )
        }
        coVerify(exactly = 0) {
            assetsRepository.add(
                walletId = any(),
                asset = any<com.wallet.core.primitives.Asset>(),
                visible = any(),
            )
        }
        coVerify { assetsRepository.updateBalances(asset.id) }
        coVerify { assetsService.syncAsset("bitcoin", Currency.USD.toJson()) }
        coVerify { streamSubscriptionService.addPrices(listOf(asset.id.toIdentifier())) }
    }

    @Test
    fun syncAssetInfo_skipsAddWhenCurrentWalletAssetAlreadyExists() = runTest {
        val wallet = mockWallet(
            id = "wallet-1",
            accounts = listOf(mockAccount(chain = Chain.Bitcoin, address = "bc1-current")),
        )
        val currentWalletAsset = mockAssetInfo(
            asset = asset,
            walletId = mockWalletId(),
            owner = mockAccount(chain = Chain.Bitcoin, address = "bc1-current"),
        ).copy(metadata = assetMetadata)

        every { assetsRepository.getAssetInfo(asset.id) } returns flowOf(currentWalletAsset)
        coEvery { assetsService.syncAsset("bitcoin", Currency.USD.toJson()) } returns assetFull.toJson()

        subject.syncAssetInfo(asset.id, wallet)

        coVerify(exactly = 0) {
            assetsRepository.linkAssetToWallet(
                walletId = any(),
                assetId = any(),
                visible = any(),
            )
        }
        coVerify { assetsRepository.updateBalances(asset.id) }
        coVerify { assetsService.syncAsset("bitcoin", Currency.USD.toJson()) }
        coVerify { streamSubscriptionService.addPrices(listOf(asset.id.toIdentifier())) }
    }

    @Test
    fun syncAssetInfo_prefetchesAssociatedAssets() = runTest {
        val associatedAssetId = mockAssetEthereum().id
        val assetFull = assetFull.copy(
            associations = listOf(
                AssetAssociation(associatedAssetId, AssetAssociationType.Official),
            ),
        )
        val wallet = mockWallet(
            accounts = listOf(mockAccount(chain = Chain.Bitcoin)),
        )

        every { assetsRepository.getAssetInfo(asset.id) } returns flowOf(mockAssetInfo(asset = asset))
        coEvery { assetsService.syncAsset("bitcoin", Currency.USD.toJson()) } returns assetFull.toJson()

        subject.syncAssetInfo(asset.id, wallet)

        coVerify { prefetchAssets.prefetchAssets(listOf(associatedAssetId)) }
    }
}
