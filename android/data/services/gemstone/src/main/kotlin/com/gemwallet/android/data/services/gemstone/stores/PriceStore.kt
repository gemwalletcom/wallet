package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.ext.toCurrency
import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.PricesDao
import com.gemwallet.android.data.service.store.database.entities.DbPrice
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.ext.secondsToMillis
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.AssetMarket
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.FiatRate
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.firstOrNull
import com.gemwallet.android.ext.toIdentifier
import uniffi.gemstone.GemAssetPrice
import uniffi.gemstone.GemPriceStore
import uniffi.gemstone.GemPriceUpdate
import com.wallet.core.primitives.AssetId

class GemstonePriceStore(
    private val pricesDao: PricesDao,
    private val assetsDao: AssetsDao,
) : GemPriceStore {

    override suspend fun getPrices(assetIds: List<String>): List<GemAssetPrice> =
        pricesDao.getByAssets(assetIds).map { it.toGemAssetPrice() }

    override suspend fun getRate(currency: String): String? =
        pricesDao.getRates(currency.toCurrency()).firstOrNull()?.toDTO()?.toJson()

    override suspend fun saveRates(rates: List<String>) =
        pricesDao.setRates(rates.map { it.decodeJson<FiatRate>().toRecord() })

    override suspend fun savePrices(currency: String, prices: List<GemPriceUpdate>) {
        val currency = currency.toCurrency()
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
        pricesDao.updateValues(currency.toCurrency(), rate)

    override suspend fun saveMarket(assetId: String, market: String) {
        assetsDao.setMarket(market.decodeJson<AssetMarket>().toRecord(AssetId(assetId)))
    }

    fun observeUsdPrice(assetId: AssetId): Flow<Double?> = pricesDao.getUsdPrice(assetId.toIdentifier())
}
