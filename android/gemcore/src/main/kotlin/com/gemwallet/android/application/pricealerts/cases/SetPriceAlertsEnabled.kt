package com.gemwallet.android.application.pricealerts.cases

interface SetPriceAlertsEnabled {
    suspend operator fun invoke(enabled: Boolean)
}
