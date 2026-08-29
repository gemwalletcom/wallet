package com.gemwallet.android.application.pricealerts.cases

import com.wallet.core.primitives.AssetId

interface SetAssetPriceAlertEnabled {
    suspend fun setAssetPriceAlertEnabled(assetId: AssetId, enabled: Boolean)
}
