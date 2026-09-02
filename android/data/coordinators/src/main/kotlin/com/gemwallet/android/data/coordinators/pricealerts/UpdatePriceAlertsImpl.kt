package com.gemwallet.android.data.coordinators.pricealerts

import com.gemwallet.android.application.pricealerts.cases.UpdatePriceAlerts
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.AssetId
import uniffi.gemstone.GemPriceAlertService
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

class UpdatePriceAlertsImpl(
    private val priceAlertService: GemPriceAlertService,
) : UpdatePriceAlerts {

    override suspend fun update() = withContext(Dispatchers.IO) { priceAlertService.sync(null) }

    override suspend fun update(assetId: AssetId) = withContext(Dispatchers.IO) { priceAlertService.sync(assetId.toIdentifier()) }
}
