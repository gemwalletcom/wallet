package com.gemwallet.android.data.coordinators.pricealerts

import com.gemwallet.android.application.pricealerts.coordinators.HasAssetPriceAlerts
import com.gemwallet.android.data.repositories.pricealerts.PriceAlertRepository
import com.wallet.core.primitives.AssetId

class HasAssetPriceAlertsImpl(
    private val priceAlertRepository: PriceAlertRepository,
) : HasAssetPriceAlerts {
    override suspend fun invoke(assetId: AssetId): Boolean =
        priceAlertRepository.hasAssetPriceAlerts(assetId)
}
