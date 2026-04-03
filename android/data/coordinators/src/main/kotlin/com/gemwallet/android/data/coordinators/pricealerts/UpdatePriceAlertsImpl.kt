package com.gemwallet.android.data.coordinators.pricealerts

import com.gemwallet.android.application.pricealerts.coordinators.UpdatePriceAlerts
import com.gemwallet.android.data.repositories.pricealerts.PriceAlertRepository
import com.gemwallet.android.data.services.gemapi.GemDeviceApiClient
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.AssetId

class UpdatePriceAlertsImpl(
    private val gemDeviceApiClient: GemDeviceApiClient,
    private val priceAlertRepository: PriceAlertRepository,
) : UpdatePriceAlerts {

    override suspend fun update() {
        val alerts = gemDeviceApiClient.getPriceAlerts()
        priceAlertRepository.updatePriceAlerts(alerts)
    }

    override suspend fun update(assetId: AssetId) {
        val alerts = gemDeviceApiClient.getPriceAlerts(assetId.toIdentifier())
        priceAlertRepository.updateAssetPriceAlerts(assetId, alerts)
    }
}
