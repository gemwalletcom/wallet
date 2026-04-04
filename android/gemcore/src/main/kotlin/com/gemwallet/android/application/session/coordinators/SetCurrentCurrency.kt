package com.gemwallet.android.application.session.coordinators

import com.wallet.core.primitives.Currency

interface SetCurrentCurrency {
    fun setCurrentCurrency(currency: Currency)
}
