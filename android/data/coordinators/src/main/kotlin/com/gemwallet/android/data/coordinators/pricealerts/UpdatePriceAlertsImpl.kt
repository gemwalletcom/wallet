package com.gemwallet.android.data.coordinators.pricealerts

import com.gemwallet.android.application.pricealerts.coordinators.UpdatePriceAlerts
import com.gemwallet.android.data.repositories.pricealerts.PriceAlertRepository
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.AssetId
import uniffi.gemstone.GemPriceAlertService
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.PriceAlert

class UpdatePriceAlertsImpl(
    private val priceAlertService: GemPriceAlertService,
    private val priceAlertRepository: PriceAlertRepository,
) : UpdatePriceAlerts {

    override suspend fun update() {
        val alerts = priceAlertService.getPriceAlerts(null).map { it.decodeJson<PriceAlert>() }
        priceAlertRepository.updatePriceAlerts(alerts)
    }

    override suspend fun update(assetId: AssetId) {
        val alerts = priceAlertService.getPriceAlerts(assetId.toIdentifier()).map { it.decodeJson<PriceAlert>() }
        priceAlertRepository.updateAssetPriceAlerts(assetId, alerts)
    }
}
