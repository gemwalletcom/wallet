package com.gemwallet.android.application.session.cases

import com.wallet.core.primitives.Currency
import kotlinx.coroutines.flow.StateFlow

interface GetCurrentCurrency {
    fun getCurrency(): StateFlow<Currency>
}
