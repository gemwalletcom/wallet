package com.gemwallet.android.application.session.cases

import com.wallet.core.primitives.Currency

interface SetCurrentCurrency {
    suspend fun setCurrentCurrency(currency: Currency)
}
