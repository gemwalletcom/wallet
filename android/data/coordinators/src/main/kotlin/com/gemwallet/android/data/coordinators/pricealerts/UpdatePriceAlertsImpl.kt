package com.gemwallet.android.data.coordinators.pricealerts

import com.gemwallet.android.application.pricealerts.cases.UpdatePriceAlerts
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.AssetId
import uniffi.gemstone.GemPriceAlertService

class UpdatePriceAlertsImpl(
    private val priceAlertService: GemPriceAlertService,
) : UpdatePriceAlerts {

    override suspend fun update() = priceAlertService.sync(null)

    override suspend fun update(assetId: AssetId) = priceAlertService.sync(assetId.toIdentifier())
}
