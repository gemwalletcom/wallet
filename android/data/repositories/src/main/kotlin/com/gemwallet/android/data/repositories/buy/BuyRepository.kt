package com.gemwallet.android.data.repositories.buy

import com.gemwallet.android.application.fiat.coordinators.GetFiatTransactions
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.service.store.database.FiatTransactionsDao
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.data.services.gemapi.GemApiClient
import com.gemwallet.android.data.services.gemapi.GemDeviceApiClient
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.FiatQuote
import com.wallet.core.primitives.FiatQuoteType
import com.wallet.core.primitives.FiatTransactionInfo
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import okio.IOException
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class BuyRepository @Inject constructor(
    private val configStore: com.gemwallet.android.data.service.store.ConfigStore,
    private val gemApi: GemApiClient,
    private val gemDeviceApiClient: GemDeviceApiClient,
    private val assetsRepository: AssetsRepository,
    private val fiatTransactionsDao: FiatTransactionsDao,
    private val getFiatTransactions: GetFiatTransactions,
) {

    suspend fun sync() {
        runCatching {
            val fiatVersion = gemApi.getConfig().versions.fiatOnRampAssets
            val currentVersion = configStore.getInt(ConfigKey.FiatAssetsVersion.string)
            if (currentVersion > 0 && currentVersion >= fiatVersion) return
            val availableToBuyIds = gemApi.getOnRampAssets()
            val availableToSellIds = gemApi.getOffRampAssets()
            val latestVersion = availableToBuyIds.version.toInt()
            assetsRepository.updateBuyAvailable(availableToBuyIds.assetIds)
            assetsRepository.updateSellAvailable(availableToSellIds.assetIds)
            configStore.putInt(ConfigKey.FiatAssetsVersion.string, latestVersion)
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
        } catch (err: Throwable) {
            throw Exception("Quotes not found", err)
        }
    }

    suspend fun getQuoteUrl(quoteId: String, walletId: String): String? {
        return try {
            gemDeviceApiClient.getFiatQuoteUrl(
                walletId = walletId,
                quoteId = quoteId,
            )?.redirectUrl
        } catch (_: Throwable) {
            null
        }
    }

    fun observeFiatTransactions(walletId: String): Flow<List<FiatTransactionInfo>> {
        return fiatTransactionsDao.getFiatTransactions(walletId).map { it.toDTO() }
    }

    suspend fun updateFiatTransactions(walletId: String) {
        try {
            val transactions = getFiatTransactions.getFiatTransactions(walletId)
            fiatTransactionsDao.insert(transactions.toRecord(walletId))
        } catch (_: Throwable) { }
    }

    private enum class ConfigKey(val string: String) {
        FiatAssetsVersion("fiat-assets-version"),
        FiatAssets("fiat-assets"),
    }
}
