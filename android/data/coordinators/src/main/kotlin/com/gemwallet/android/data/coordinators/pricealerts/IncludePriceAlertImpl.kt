package com.gemwallet.android.data.coordinators.pricealerts

import com.gemwallet.android.application.pricealerts.coordinators.IncludePriceAlert
import com.gemwallet.android.cases.device.SyncDeviceInfo
import com.gemwallet.android.data.repositories.pricealerts.PriceAlertRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.services.gemapi.GemDeviceApiClient
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PriceAlert
import com.wallet.core.primitives.PriceAlertDirection
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.flow.firstOrNull

class IncludePriceAlertImpl(
    private val gemDeviceApiClient: GemDeviceApiClient,
    private val sessionRepository: SessionRepository,
    private val priceAlertRepository: PriceAlertRepository,
    private val syncDeviceInfo: SyncDeviceInfo,
) : IncludePriceAlert {

    override suspend fun invoke(
        assetId: AssetId,
        currency: Currency?,
        price: Double?,
        percentage: Double?,
        direction: PriceAlertDirection?
    ) {
        val currency = currency?.string ?: sessionRepository.getCurrentCurrency().string
        val priceAlert = PriceAlert(
            assetId = assetId,
            currency = currency,
            price = price,
            pricePercentChange = percentage,
            priceDirection = direction,
        )
        priceAlertRepository.getSamePriceAlert(priceAlert)?.let {
            priceAlertRepository.enable(it.id)
        } ?: priceAlertRepository.addPriceAlert(priceAlert)
        enablePriceAlertsIfNeeded()

        try {
            gemDeviceApiClient.includePriceAlert(alerts = listOf(priceAlert))
        } catch (_: Exception) {
            currentCoroutineContext().ensureActive()
        }
    }

    private suspend fun enablePriceAlertsIfNeeded() {
        if (priceAlertRepository.isPriceAlertsEnabled().firstOrNull() == true) {
            return
        }

        priceAlertRepository.togglePriceAlerts(true)
        try {
            syncDeviceInfo.syncDeviceInfo()
        } catch (_: Exception) {
            currentCoroutineContext().ensureActive()
        }
    }
}
