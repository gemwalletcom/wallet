package com.gemwallet.android.data.coordinators.pricealerts

import com.gemwallet.android.application.pricealerts.coordinators.SetPriceAlertsEnabled
import com.gemwallet.android.cases.device.SyncDeviceInfo
import com.gemwallet.android.data.repositories.pricealerts.PriceAlertRepository
import kotlinx.coroutines.flow.firstOrNull

class SetPriceAlertsEnabledImpl(
    private val priceAlertRepository: PriceAlertRepository,
    private val syncDeviceInfo: SyncDeviceInfo,
) : SetPriceAlertsEnabled {

    override suspend fun invoke(enabled: Boolean) {
        if (enabled && priceAlertRepository.isPriceAlertsEnabled().firstOrNull() == true) {
            return
        }

        priceAlertRepository.togglePriceAlerts(enabled)
        syncDeviceInfo.syncDeviceInfo()
    }
}
