package com.gemwallet.android.data.repositories.buy

import com.gemwallet.android.application.assets.coordinators.PrefetchAssets
import com.gemwallet.android.application.config.coordinators.GetRemoteConfig
import com.gemwallet.android.application.fiat.coordinators.GetFiatTransactions
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.service.store.ConfigStore
import com.gemwallet.android.data.service.store.database.FiatTransactionsDao
import com.gemwallet.android.data.services.gemapi.GemDeviceApiClient
import com.gemwallet.android.testkit.mockAssetEthereum
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.ConfigResponse
import com.wallet.core.primitives.ConfigVersions
import com.wallet.core.primitives.FiatAssets
import com.wallet.core.primitives.FiatProviderName
import com.wallet.core.primitives.FiatQuoteType
import com.wallet.core.primitives.FiatTransaction
import com.wallet.core.primitives.FiatTransactionData
import com.wallet.core.primitives.FiatTransactionStatus
import com.wallet.core.primitives.SwapConfig
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.test.runTest
import org.junit.Test

class BuyRepositoryTest {

    private val configStore = mockk<ConfigStore>(relaxed = true)
    private val getRemoteConfig = mockk<GetRemoteConfig>()
    private val gemDeviceApiClient = mockk<GemDeviceApiClient>()
    private val assetsRepository = mockk<AssetsRepository>(relaxed = true)
    private val assetsCoordinator = mockk<PrefetchAssets>(relaxed = true)
    private val fiatTransactionsDao = mockk<FiatTransactionsDao>(relaxed = true)
    private val getFiatTransactions = mockk<GetFiatTransactions>(relaxed = true)

    private val subject = BuyRepository(
        configStore = configStore,
        getRemoteConfig = getRemoteConfig,
        gemDeviceApiClient = gemDeviceApiClient,
        assetsRepository = assetsRepository,
        assetsCoordinator = assetsCoordinator,
        fiatTransactionsDao = fiatTransactionsDao,
        getFiatTransactions = getFiatTransactions,
    )

    @Test
    fun sync_usesRemoteConfigToRefreshBuyAndSellAssets() = runTest {
        coEvery { getRemoteConfig.getRemoteConfig() } returns remoteConfig(
            fiatOnRampAssets = 2,
            fiatOffRampAssets = 3,
        )
        every { configStore.getInt("fiat-on-ramp-assets-version", "") } returns 1
        every { configStore.getInt("fiat-off-ramp-assets-version", "") } returns 2
        coEvery { gemDeviceApiClient.getBuyableFiatAssets() } returns FiatAssets(5u, listOf("bitcoin"))
        coEvery { gemDeviceApiClient.getSellableFiatAssets() } returns FiatAssets(7u, listOf("ethereum"))

        subject.sync()

        coVerify { getRemoteConfig.getRemoteConfig() }
        coVerify { assetsRepository.updateBuyAvailable(listOf("bitcoin")) }
        coVerify { assetsRepository.updateSellAvailable(listOf("ethereum")) }
        verify { configStore.putInt("fiat-on-ramp-assets-version", 5, "") }
        verify { configStore.putInt("fiat-off-ramp-assets-version", 7, "") }
    }

    @Test
    fun sync_skipsRefreshWhenVersionsAreCurrent() = runTest {
        coEvery { getRemoteConfig.getRemoteConfig() } returns remoteConfig(
            fiatOnRampAssets = 2,
            fiatOffRampAssets = 3,
        )
        every { configStore.getInt("fiat-on-ramp-assets-version", "") } returns 2
        every { configStore.getInt("fiat-off-ramp-assets-version", "") } returns 3

        subject.sync()

        coVerify(exactly = 0) { gemDeviceApiClient.getBuyableFiatAssets() }
        coVerify(exactly = 0) { gemDeviceApiClient.getSellableFiatAssets() }
        coVerify(exactly = 0) { assetsRepository.updateBuyAvailable(any()) }
        coVerify(exactly = 0) { assetsRepository.updateSellAvailable(any()) }
    }

    @Test
    fun updateFiatTransactions_prefetchesDistinctAssetIds() = runTest {
        val wallet = mockWallet(id = "wallet-1")
        val ethereum = mockAssetEthereum()
        val transaction = FiatTransactionData(
            transaction = FiatTransaction(
                id = "tx-1",
                assetId = ethereum.id,
                transactionType = FiatQuoteType.Buy,
                provider = FiatProviderName.MoonPay,
                status = FiatTransactionStatus.Pending,
                fiatAmount = 100.0,
                fiatCurrency = "USD",
                value = "1000000000000000000",
                createdAt = 1,
            ),
        )

        coEvery { getFiatTransactions.getFiatTransactions(wallet.id) } returns listOf(transaction, transaction)

        subject.updateFiatTransactions(wallet)

        coVerify { assetsCoordinator.prefetchAssets(listOf(ethereum.id)) }
        verify { fiatTransactionsDao.insert(any()) }
    }

    private fun remoteConfig(
        fiatOnRampAssets: Int,
        fiatOffRampAssets: Int,
    ) = ConfigResponse(
        releases = emptyList(),
        versions = ConfigVersions(
            fiatOnRampAssets = fiatOnRampAssets,
            fiatOffRampAssets = fiatOffRampAssets,
            swapAssets = 0,
        ),
        swap = SwapConfig(
            enabledProviders = emptyList(),
        ),
    )
}
