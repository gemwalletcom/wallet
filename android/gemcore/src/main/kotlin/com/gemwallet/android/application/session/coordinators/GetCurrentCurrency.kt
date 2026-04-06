package com.gemwallet.android.application.session.coordinators

import com.wallet.core.primitives.Currency
import kotlinx.coroutines.flow.Flow

interface GetCurrentCurrency {
    suspend fun getCurrentCurrency(): Currency

    fun getCurrency(): Flow<Currency>
}
