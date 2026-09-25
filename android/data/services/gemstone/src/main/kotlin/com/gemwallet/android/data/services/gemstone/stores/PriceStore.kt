package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.data.services.store.database.AssetsDao
import com.gemwallet.android.data.services.store.database.PricesDao
import com.gemwallet.android.data.services.store.database.entities.DbPrice
import com.gemwallet.android.data.services.store.database.entities.toDTO
import com.gemwallet.android.data.services.store.database.entities.toRecord
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.toPrimitives
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Currency
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.firstOrNull
import uniffi.gemstone.AssetPrice
import uniffi.gemstone.GemPriceStore
import uniffi.gemstone.GemPriceUpdate

class GemstonePriceStore(private val pricesDao: PricesDao, private val assetsDao: AssetsDao) : GemPriceStore {

    override suspend fun getPrices(assetIds: List<String>): List<AssetPrice> = pricesDao.getByAssets(assetIds).map { it.toAssetPrice() }

    override suspend fun getRate(currency: uniffi.gemstone.Currency): uniffi.gemstone.FiatRate? = pricesDao.getRates(currency.toPrimitives()).firstOrNull()?.toDTO()?.toGem()

    override suspend fun getRates(): List<uniffi.gemstone.FiatRate> = pricesDao.getRates().toDTO().map { it.toGem() }

    override suspend fun saveRatesAndPrices(currency: uniffi.gemstone.Currency, rates: List<uniffi.gemstone.FiatRate>, conversion: uniffi.gemstone.FiatRate?, prices: List<GemPriceUpdate>) = pricesDao.saveRatesAndPrices(
        rates = rates.map { it.toPrimitives().toRecord() },
        conversion = conversion?.toPrimitives()?.toRecord(),
        prices = prices.map { it.toRecord(currency.toPrimitives()) },
    )

    override suspend fun savePrices(currency: uniffi.gemstone.Currency, prices: List<GemPriceUpdate>) {
        pricesDao.insert(prices.map { it.toRecord(currency.toPrimitives()) })
    }

    override suspend fun convertPrices(currency: uniffi.gemstone.Currency, rate: Double) = pricesDao.updateValues(currency.toPrimitives(), rate)

    override suspend fun saveMarket(assetId: String, market: uniffi.gemstone.AssetMarket) {
        assetsDao.setMarket(market.toPrimitives().toRecord(AssetId(assetId)))
    }

    fun observeUsdPrice(assetId: AssetId): Flow<Double?> = pricesDao.getUsdPrice(assetId.toIdentifier())
}

private fun GemPriceUpdate.toRecord(currency: Currency) = DbPrice(
    assetId = assetId,
    value = price,
    usdValue = priceUsd,
    dayChanged = priceChangePercentage24h,
    currency = currency,
    updatedAt = updatedAt,
)
