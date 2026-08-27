package com.gemwallet.android.data.repositories.prices

import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.AssetBasic
import com.wallet.core.primitives.AssetPrice
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.WebSocketPricePayload
import javax.inject.Inject
import javax.inject.Singleton
import uniffi.gemstone.GemPriceService

@Singleton
class PricesRepository @Inject constructor(
    private val priceService: GemPriceService,
    private val sessionRepository: SessionRepository,
) {

    suspend fun updatePrices(payload: WebSocketPricePayload) {
        val currency = sessionRepository.getCurrentCurrency().toJson()
        priceService.updateRates(payload.rates.map { it.toJson() }, currency)
        priceService.updatePrices(payload.prices.map { it.toJson() }, currency)
    }

    suspend fun updatePrices(assets: List<AssetBasic>, currency: Currency) {
        val prices = assets.mapNotNull { asset -> asset.price?.let { AssetPrice(asset.asset.id, it.price, it.priceChangePercentage24h, it.updatedAt) } }
        priceService.updatePrices(prices.map { it.toJson() }, currency.toJson())
    }

    suspend fun convertPricesToCurrency(currency: Currency) {
        priceService.changeCurrency(currency.toJson())
    }
}
