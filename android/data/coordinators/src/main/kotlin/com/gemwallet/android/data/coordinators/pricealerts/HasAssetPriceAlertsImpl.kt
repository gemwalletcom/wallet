package com.gemwallet.android.data.coordinators.pricealerts

import com.gemwallet.android.application.pricealerts.cases.HasAssetPriceAlerts
import com.gemwallet.android.data.adapters.gemstone.GemstonePriceAlertStore
import com.wallet.core.primitives.AssetId

class HasAssetPriceAlertsImpl(
    private val priceAlertStore: GemstonePriceAlertStore,
) : HasAssetPriceAlerts {

    override suspend fun invoke(assetId: AssetId): Boolean = priceAlertStore.hasAssetPriceAlerts(assetId)
}
