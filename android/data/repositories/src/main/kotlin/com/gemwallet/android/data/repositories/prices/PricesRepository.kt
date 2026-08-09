package com.gemwallet.android.data.repositories.prices

import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.service.store.database.PricesDao
import com.gemwallet.android.data.service.store.database.entities.DbPrice
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.data.service.store.database.entities.toPriceRecord
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.AssetBasic
import com.wallet.core.primitives.AssetFull
import com.wallet.core.primitives.AssetPrice
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.FiatRate
import com.wallet.core.primitives.WebSocketPricePayload
import kotlinx.coroutines.flow.firstOrNull
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class PricesRepository @Inject constructor(
    private val pricesDao: PricesDao,
    private val sessionRepository: SessionRepository,
) {

    suspend fun updatePrices(payload: WebSocketPricePayload) {
        val currency = sessionRepository.getCurrentCurrency()
        updateRates(payload.rates, currency)
        val rate = currentRate(currency) ?: return
        pricesDao.insert(payload.prices.toRecord(rate))
    }

    suspend fun updatePrice(price: AssetPrice) {
        val currency = sessionRepository.getCurrentCurrency()
        val rate = currentRate(currency) ?: return
        pricesDao.insert(price.toRecord(rate))
    }

    suspend fun updatePrices(assets: List<AssetBasic>, currency: Currency) {
        val rate = currentRate(currency) ?: return
        val prices = assets.toPriceRecord(rate)
        if (prices.isNotEmpty()) {
            pricesDao.insert(prices)
        }
    }

    suspend fun updatePrice(assetFull: AssetFull, rate: FiatRate) {
        pricesDao.insert(
            assetFull.toPriceRecord(rate)
                ?: DbPrice(assetId = assetFull.asset.id.toIdentifier(), currency = rate.symbol)
        )
    }

    suspend fun convertPricesToCurrency(currency: Currency) {
        val rate = currentRate(currency) ?: return
        pricesDao.getAll().firstOrNull()?.map {
            it.copy(value = (it.usdValue ?: 0.0) * rate.rate, currency = currency)
        }?.let { pricesDao.insert(it) }
    }

    private suspend fun currentRate(currency: Currency): FiatRate? {
        return pricesDao.getRates(currency).firstOrNull()?.toDTO()
    }

    private suspend fun updateRates(newRates: List<FiatRate>, currency: Currency) {
        pricesDao.setRates(newRates.toRecord())
        newRates.firstOrNull { it.symbol == currency }?.let { rate ->
            pricesDao.updateValues(currency, rate.rate)
        }
    }
}
