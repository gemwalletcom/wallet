package com.gemwallet.android.data.repositories.buy

import com.gemwallet.android.application.assets.coordinators.PrefetchAssets
import com.gemwallet.android.application.config.coordinators.GetRemoteConfig
import com.gemwallet.android.application.fiat.coordinators.GetFiatTransactions
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.service.store.database.FiatTransactionsDao
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.data.services.gemapi.GemDeviceApiClient
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.FiatQuote
import com.wallet.core.primitives.FiatQuoteType
import com.wallet.core.primitives.FiatTransactionAssetData
import com.wallet.core.primitives.FiatTransactionData
import com.wallet.core.primitives.Wallet
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import okio.IOException
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class BuyRepository @Inject constructor(
    private val configStore: com.gemwallet.android.data.service.store.ConfigStore,
    private val getRemoteConfig: GetRemoteConfig,
    private val gemDeviceApiClient: GemDeviceApiClient,
    private val assetsRepository: AssetsRepository,
    private val assetsCoordinator: PrefetchAssets,
    private val fiatTransactionsDao: FiatTransactionsDao,
    private val getFiatTransactions: GetFiatTransactions,
) {

    suspend fun sync() {
        try {
            val versions = getRemoteConfig.getRemoteConfig().versions
            val currentBuyVersion = configStore.getInt(ConfigKey.FiatOnRampAssetsVersion.string)
            val currentSellVersion = configStore.getInt(ConfigKey.FiatOffRampAssetsVersion.string)
            val shouldSyncBuyAssets = currentBuyVersion <= 0 || currentBuyVersion < versions.fiatOnRampAssets
            val shouldSyncSellAssets = currentSellVersion <= 0 || currentSellVersion < versions.fiatOffRampAssets

            if (!shouldSyncBuyAssets && !shouldSyncSellAssets) {
                return
            }

            if (shouldSyncBuyAssets) {
                val availableToBuyIds = gemDeviceApiClient.getBuyableFiatAssets()
                assetsRepository.updateBuyAvailable(availableToBuyIds.assetIds)
                configStore.putInt(ConfigKey.FiatOnRampAssetsVersion.string, availableToBuyIds.version.toInt())
            }
            if (shouldSyncSellAssets) {
                val availableToSellIds = gemDeviceApiClient.getSellableFiatAssets()
                assetsRepository.updateSellAvailable(availableToSellIds.assetIds)
                configStore.putInt(ConfigKey.FiatOffRampAssetsVersion.string, availableToSellIds.version.toInt())
            }
        } catch (_: Exception) {
            currentCoroutineContext().ensureActive()
        }
    }

    suspend fun getQuotes(
        walletId: String,
        asset: Asset,
        type: FiatQuoteType,
        fiatCurrency: String,
        amount: Double,
    ): List<FiatQuote> {
        return try {
            gemDeviceApiClient.getFiatQuotes(
                assetId = asset.id.toIdentifier(),
                amount = amount,
                currency = fiatCurrency,
                walletId = walletId,
                type = type.string,
            )?.quotes ?: throw IOException()
        } catch (err: Exception) {
            currentCoroutineContext().ensureActive()
            throw Exception("Quotes not found", err)
        }
    }

    suspend fun getQuoteUrl(quoteId: String, walletId: String): String? {
        return try {
            gemDeviceApiClient.getFiatQuoteUrl(
                walletId = walletId,
                quoteId = quoteId,
            )?.redirectUrl
        } catch (_: Exception) {
            currentCoroutineContext().ensureActive()
            null
        }
    }

    fun observeFiatTransactions(walletId: String): Flow<List<FiatTransactionAssetData>> {
        return fiatTransactionsDao.getFiatTransactions(walletId).map { it.toDTO() }
    }

    suspend fun updateFiatTransactions(wallet: Wallet) {
        try {
            val transactions = getFiatTransactions.getFiatTransactions(wallet.id)
            prefetchAssets(transactions)
            fiatTransactionsDao.insert(transactions.toRecord(wallet.id))
        } catch (_: Exception) {
            currentCoroutineContext().ensureActive()
        }
    }

    private suspend fun prefetchAssets(transactions: List<FiatTransactionData>) {
        val assetIds = transactions
            .map { it.transaction.assetId }
            .distinct()

        assetsCoordinator.prefetchAssets(assetIds)
    }

    private enum class ConfigKey(val string: String) {
        FiatOnRampAssetsVersion("fiat-on-ramp-assets-version"),
        FiatOffRampAssetsVersion("fiat-off-ramp-assets-version"),
    }
}
