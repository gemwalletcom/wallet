package com.gemwallet.android.application.pricealerts.cases

import kotlinx.coroutines.flow.Flow

interface GetPriceAlertsEnabled {
    fun isPriceAlertsEnabled(): Flow<Boolean>
}
