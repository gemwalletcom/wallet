package com.gemwallet.android.data.repositories.gemstone

import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.PricesDao
import com.gemwallet.android.data.service.store.database.entities.DbPrice
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.ext.secondsToMillis
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.ext.toAssetId
import com.wallet.core.primitives.AssetMarket
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.FiatRate
import kotlinx.coroutines.flow.firstOrNull
import dagger.Lazy
import com.gemwallet.android.data.repositories.pricealerts.PriceAlertRepository
import com.gemwallet.android.ext.toIdentifier
import uniffi.gemstone.GemPriceStore
import uniffi.gemstone.GemPriceUpdate

class GemstonePriceStore(
    private val pricesDao: PricesDao,
    private val assetsDao: AssetsDao,
    private val priceAlertRepository: Lazy<PriceAlertRepository>,
) : GemPriceStore {

    override suspend fun getEnabledPriceAssetIds(walletId: String): List<String> {
        val enabled = assetsDao.getAssetsPriceUpdate(walletId).mapNotNull { it.toAssetId()?.toIdentifier() }
        val alerts = priceAlertRepository.get().getPriceAlertAssetIds().firstOrNull().orEmpty().map { it.toIdentifier() }
        return (enabled + alerts).distinct()
    }

    override suspend fun getRate(currency: String): String? =
        pricesDao.getRates(currency.decodeJson<Currency>()).firstOrNull()?.toDTO()?.toJson()

    override suspend fun saveRates(rates: List<String>) =
        pricesDao.setRates(rates.map { it.decodeJson<FiatRate>().toRecord() })

    override suspend fun savePrices(currency: String, prices: List<GemPriceUpdate>) {
        val currency = currency.decodeJson<Currency>()
        pricesDao.insert(
            prices.map {
                DbPrice(
                    assetId = it.assetId,
                    value = it.price,
                    usdValue = it.priceUsd,
                    dayChanged = it.priceChangePercentage24h,
                    currency = currency,
                    updatedAt = it.updatedAt.secondsToMillis(),
                )
            }
        )
    }

    override suspend fun convertPrices(currency: String, rate: Double) =
        pricesDao.updateValues(currency.decodeJson<Currency>(), rate)

    override suspend fun saveMarket(assetId: String, market: String) {
        val id = assetId.toAssetId() ?: return
        assetsDao.setMarket(market.decodeJson<AssetMarket>().toRecord(id))
    }
}
