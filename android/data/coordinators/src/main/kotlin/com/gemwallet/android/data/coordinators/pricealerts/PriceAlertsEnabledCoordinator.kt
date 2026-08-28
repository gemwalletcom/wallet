package com.gemwallet.android.data.coordinators.pricealerts

import com.gemwallet.android.application.pricealerts.cases.GetPriceAlertsEnabled
import com.gemwallet.android.application.pricealerts.cases.SetPriceAlertsEnabled
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onStart
import uniffi.gemstone.GemPriceAlertService

class PriceAlertsEnabledCoordinator(
    private val priceAlertService: GemPriceAlertService,
) : GetPriceAlertsEnabled, SetPriceAlertsEnabled {

    private val changes = MutableSharedFlow<Unit>()

    override fun isPriceAlertsEnabled(): Flow<Boolean> = changes
        .onStart { emit(Unit) }
        .map { priceAlertService.isEnabled() }

    override suspend fun invoke(enabled: Boolean) {
        priceAlertService.setEnabled(enabled)
        changes.emit(Unit)
    }
}
