package com.gemwallet.android.data.coordinators.pricealerts

import com.gemwallet.android.application.pricealerts.coordinators.HasAssetPriceAlerts
import com.gemwallet.android.application.pricealerts.coordinators.SyncAssetPriceAlerts
import com.gemwallet.android.application.pricealerts.coordinators.UpdatePriceAlerts
import com.wallet.core.primitives.AssetId

class SyncAssetPriceAlertsImpl(
    private val hasAssetPriceAlerts: HasAssetPriceAlerts,
    private val updatePriceAlerts: UpdatePriceAlerts,
) : SyncAssetPriceAlerts {

    override suspend fun invoke(assetId: AssetId) {
        if (hasAssetPriceAlerts(assetId)) {
            runCatching { updatePriceAlerts.update(assetId) }
        }
    }
}
