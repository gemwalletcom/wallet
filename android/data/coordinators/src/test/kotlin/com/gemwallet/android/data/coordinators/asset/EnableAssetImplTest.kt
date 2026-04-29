package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.cases.tokens.SyncAssetPrices
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetEthereum
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockAssetMetaData
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Currency
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import org.junit.Test

class EnableAssetImplTest {

    private val sessionRepository = mockk<SessionRepository>(relaxed = true) {
        coEvery { getCurrentCurrency() } returns Currency.USD
    }
    private val syncAssetPrices = mockk<SyncAssetPrices>(relaxed = true)
    private val assetsRepository = mockk<AssetsRepository>(relaxed = true)

    private val subject = EnableAssetImpl(
        sessionRepository = sessionRepository,
        syncAssetPrices = syncAssetPrices,
        assetsRepository = assetsRepository,
    )

    @Test
    fun freshAsset_syncsPriceThenLinksThenUpdatesBalance() = runTest {
        val asset = mockAsset()
        every { assetsRepository.getAssetsInfo(listOf(asset.id)) } returns flowOf(emptyList())

        subject("wallet-1", asset.id)

        coVerify(ordering = io.mockk.Ordering.ORDERED) {
            syncAssetPrices(listOf(asset.id), Currency.USD)
            assetsRepository.linkAssetToWallet("wallet-1", asset.id, visible = true)
            assetsRepository.updateBalances(asset.id)
        }
    }

    @Test
    fun skipsAlreadyEnabledAsset() = runTest {
        val asset = mockAsset()
        every { assetsRepository.getAssetsInfo(listOf(asset.id)) } returns flowOf(
            listOf(
                mockAssetInfo(
                    asset = asset,
                    walletId = "wallet-1",
                    metadata = mockAssetMetaData(isBalanceEnabled = true),
                )
            )
        )

        subject("wallet-1", asset.id)

        coVerify(exactly = 0) { syncAssetPrices(any(), any()) }
        coVerify(exactly = 0) { assetsRepository.linkAssetToWallet(any(), any(), any()) }
        coVerify(exactly = 0) { assetsRepository.updateBalances(*anyVararg()) }
    }

    @Test
    fun enablesAssetLinkedButHidden() = runTest {
        val asset = mockAsset()
        every { assetsRepository.getAssetsInfo(listOf(asset.id)) } returns flowOf(
            listOf(
                mockAssetInfo(
                    asset = asset,
                    walletId = "wallet-1",
                    metadata = mockAssetMetaData(isBalanceEnabled = false),
                )
            )
        )

        subject("wallet-1", asset.id)

        coVerify(exactly = 1) { syncAssetPrices(listOf(asset.id), Currency.USD) }
        coVerify(exactly = 1) { assetsRepository.linkAssetToWallet("wallet-1", asset.id, visible = true) }
    }

    @Test
    fun batch_dedupesAndQueriesOnce_skippingAlreadyEnabled() = runTest {
        val fresh = mockAsset()
        val enabled = mockAssetEthereum()
        every { assetsRepository.getAssetsInfo(listOf(fresh.id, enabled.id)) } returns flowOf(
            listOf(
                mockAssetInfo(
                    asset = enabled,
                    walletId = "wallet-1",
                    metadata = mockAssetMetaData(isBalanceEnabled = true),
                )
            )
        )

        subject("wallet-1", listOf(fresh.id, enabled.id, fresh.id))

        coVerify(exactly = 1) { assetsRepository.getAssetsInfo(listOf(fresh.id, enabled.id)) }
        coVerify(exactly = 1) { syncAssetPrices(listOf(fresh.id), Currency.USD) }
        coVerify(exactly = 1) { assetsRepository.linkAssetToWallet("wallet-1", fresh.id, visible = true) }
        coVerify(exactly = 0) { assetsRepository.linkAssetToWallet("wallet-1", enabled.id, any()) }
        coVerify(exactly = 1) { assetsRepository.updateBalances(fresh.id) }
    }

    @Test
    fun emptyList_isNoOp() = runTest {
        subject("wallet-1", emptyList<AssetId>())

        coVerify(exactly = 0) { assetsRepository.getAssetsInfo(any<List<AssetId>>()) }
        coVerify(exactly = 0) { syncAssetPrices(any(), any()) }
        coVerify(exactly = 0) { assetsRepository.linkAssetToWallet(any(), any(), any()) }
    }
}
