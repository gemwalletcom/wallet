package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toCurrency
import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.PricesDao
import com.gemwallet.android.data.service.store.database.entities.DbPrice
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.AssetMarket
import com.wallet.core.primitives.Currency
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.firstOrNull
import com.gemwallet.android.ext.toIdentifier
import uniffi.gemstone.AssetPrice
import uniffi.gemstone.GemPriceStore
import uniffi.gemstone.GemPriceUpdate
import com.wallet.core.primitives.AssetId

class GemstonePriceStore(
    private val pricesDao: PricesDao,
    private val assetsDao: AssetsDao,
) : GemPriceStore {

    override suspend fun getPrices(assetIds: List<String>): List<AssetPrice> =
        pricesDao.getByAssets(assetIds).map { it.toAssetPrice() }

    override suspend fun getRate(currency: String): uniffi.gemstone.FiatRate? =
        pricesDao.getRates(currency.toCurrency()).firstOrNull()?.toDTO()?.toGem()

    override suspend fun saveRates(rates: List<uniffi.gemstone.FiatRate>) =
        pricesDao.setRates(rates.map { it.toPrimitives().toRecord() })

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
                    updatedAt = it.updatedAt,
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
