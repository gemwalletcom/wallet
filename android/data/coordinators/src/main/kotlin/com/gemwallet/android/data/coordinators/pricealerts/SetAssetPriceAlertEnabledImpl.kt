package com.gemwallet.android.data.coordinators.pricealerts

import com.gemwallet.android.application.pricealerts.coordinators.ExcludePriceAlert
import com.gemwallet.android.application.pricealerts.coordinators.IncludePriceAlert
import com.gemwallet.android.application.pricealerts.coordinators.SetAssetPriceAlertEnabled
import com.wallet.core.primitives.AssetId

class SetAssetPriceAlertEnabledImpl(
    private val includePriceAlert: IncludePriceAlert,
    private val excludePriceAlert: ExcludePriceAlert,
) : SetAssetPriceAlertEnabled {

    override suspend fun invoke(assetId: AssetId, enabled: Boolean) {
        if (enabled) {
            includePriceAlert(assetId)
        } else {
            excludePriceAlert(assetId)
        }
    }
}
