package com.gemwallet.android.application.pricealerts.coordinators

interface SetPriceAlertsEnabled {
    suspend operator fun invoke(enabled: Boolean)
}
