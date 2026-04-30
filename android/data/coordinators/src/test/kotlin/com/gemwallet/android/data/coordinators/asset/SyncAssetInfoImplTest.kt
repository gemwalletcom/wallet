package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.stream.StreamSubscriptionService
import com.gemwallet.android.data.services.gemapi.GemApiClient
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetFull
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockAssetLink
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.AssetMetaData
import com.wallet.core.primitives.AssetScore
import com.wallet.core.primitives.Chain
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import org.junit.Test

class SyncAssetInfoImplTest {

    private val gemApiClient = mockk<GemApiClient>()
    private val assetsRepository = mockk<AssetsRepository>(relaxed = true)
    private val streamSubscriptionService = mockk<StreamSubscriptionService>(relaxed = true)

    private val subject = SyncAssetInfoImpl(
        gemApiClient = gemApiClient,
        assetsRepository = assetsRepository,
        streamSubscriptionService = streamSubscriptionService,
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
            walletId = "wallet-2",
            owner = mockAccount(chain = Chain.Bitcoin, address = "bc1-foreign"),
        ).copy(metadata = assetMetadata)

        every { assetsRepository.getAssetInfo(asset.id) } returns flowOf(null)
        every { assetsRepository.getTokenInfo(asset.id) } returns flowOf(foreignWalletAsset)
        coEvery { gemApiClient.getAsset("bitcoin") } returns assetFull

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
        coVerify { assetsRepository.updateAssetMetadata(assetFull) }
        verify { streamSubscriptionService.addAssetIds(listOf(asset.id)) }
    }

    @Test
    fun syncAssetInfo_skipsAddWhenCurrentWalletAssetAlreadyExists() = runTest {
        val wallet = mockWallet(
            id = "wallet-1",
            accounts = listOf(mockAccount(chain = Chain.Bitcoin, address = "bc1-current")),
        )
        val currentWalletAsset = mockAssetInfo(
            asset = asset,
            walletId = "wallet-1",
            owner = mockAccount(chain = Chain.Bitcoin, address = "bc1-current"),
        ).copy(metadata = assetMetadata)

        every { assetsRepository.getAssetInfo(asset.id) } returns flowOf(currentWalletAsset)
        coEvery { gemApiClient.getAsset("bitcoin") } returns assetFull

        subject.syncAssetInfo(asset.id, wallet)

        coVerify(exactly = 0) {
            assetsRepository.linkAssetToWallet(
                walletId = any(),
                assetId = any(),
                visible = any(),
            )
        }
        coVerify { assetsRepository.updateBalances(asset.id) }
        coVerify { assetsRepository.updateAssetMetadata(assetFull) }
        verify { streamSubscriptionService.addAssetIds(listOf(asset.id)) }
    }
}
