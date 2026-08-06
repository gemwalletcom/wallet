package com.gemwallet.android.data.repositories.prices

import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.service.store.database.PricesDao
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.data.service.store.database.entities.toRecord
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
        val rate = pricesDao.getRates(currency).firstOrNull()?.toDTO() ?: return
        pricesDao.insert(payload.prices.toRecord(rate))
    }

    private suspend fun updateRates(newRates: List<FiatRate>, currency: Currency) {
        pricesDao.setRates(newRates.toRecord())
        newRates.firstOrNull { it.symbol == currency.string }?.let { rate ->
            pricesDao.updateValues(currency.string, rate.rate)
        }
    }
}
