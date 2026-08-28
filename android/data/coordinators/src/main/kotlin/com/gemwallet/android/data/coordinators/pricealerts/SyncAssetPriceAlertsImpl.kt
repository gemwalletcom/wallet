package com.gemwallet.android.data.coordinators.pricealerts

import android.util.Log
import com.gemwallet.android.application.pricealerts.coordinators.HasAssetPriceAlerts
import com.gemwallet.android.application.pricealerts.coordinators.SyncAssetPriceAlerts
import com.gemwallet.android.application.pricealerts.coordinators.UpdatePriceAlerts
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.AssetId

class SyncAssetPriceAlertsImpl(
    private val hasAssetPriceAlerts: HasAssetPriceAlerts,
    private val updatePriceAlerts: UpdatePriceAlerts,
) : SyncAssetPriceAlerts {

    override suspend fun invoke(assetId: AssetId) {
        if (hasAssetPriceAlerts(assetId)) {
            runCatchingCancellable { updatePriceAlerts.update(assetId) }
                .onFailure { Log.e(TAG, "price alerts sync failed for ${assetId.toIdentifier()}", it) }
        }
    }

    private companion object {
        const val TAG = "SyncAssetPriceAlerts"
    }
}
