package com.gemwallet.android.application.pricealerts.coordinators

import kotlinx.coroutines.flow.Flow

interface GetPriceAlertsEnabled {
    fun isPriceAlertsEnabled(): Flow<Boolean>
}
