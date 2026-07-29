package com.gemwallet.android.data.coordinators.pricealerts

import com.gemwallet.android.application.pricealerts.coordinators.GetPriceAlertsEnabled
import com.gemwallet.android.data.repositories.pricealerts.PriceAlertRepository
import kotlinx.coroutines.flow.Flow

class GetPriceAlertsEnabledImpl(
    private val priceAlertRepository: PriceAlertRepository,
) : GetPriceAlertsEnabled {

    override fun isPriceAlertsEnabled(): Flow<Boolean> = priceAlertRepository.isPriceAlertsEnabled()
}
