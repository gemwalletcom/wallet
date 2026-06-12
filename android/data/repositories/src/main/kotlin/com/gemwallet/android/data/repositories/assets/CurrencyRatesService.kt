package com.gemwallet.android.data.repositories.assets

import com.gemwallet.android.data.service.store.database.PricesDao
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.FiatRate
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.map
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class CurrencyRatesService @Inject constructor(
    private val pricesDao: PricesDao,
) {

    suspend fun changeCurrency(currency: Currency) {
        val rate = pricesDao.getRates(currency).map { it?.toDTO() }.firstOrNull() ?: return
        pricesDao.getAll().firstOrNull()?.map {
            it.copy(value = (it.usdValue ?: 0.0) * rate.rate, currency = currency.string)
        }?.let { pricesDao.insert(it) }
    }

    fun getCurrencyRate(currency: Currency): Flow<FiatRate?> {
        return pricesDao.getRates(currency).map { it?.toDTO() }
    }
}
