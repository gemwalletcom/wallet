package com.gemwallet.android.data.repositories.prices

import com.gemwallet.android.data.service.store.database.PricesDao
import com.gemwallet.android.data.service.store.database.entities.DbPrice
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.FiatRate
import kotlinx.coroutines.flow.firstOrNull
import uniffi.gemstone.GemPriceStore
import uniffi.gemstone.GemPriceUpdate

class GemstonePriceStore(
    private val pricesDao: PricesDao,
) : GemPriceStore {

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
                    updatedAt = it.updatedAt,
                )
            }
        )
    }

    override suspend fun convertPrices(currency: String, rate: Double) =
        pricesDao.updateValues(currency.decodeJson<Currency>(), rate)
}
