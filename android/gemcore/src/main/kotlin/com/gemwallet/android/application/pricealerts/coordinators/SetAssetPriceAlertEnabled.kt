package com.gemwallet.android.application.pricealerts.coordinators

import com.wallet.core.primitives.AssetId

interface SetAssetPriceAlertEnabled {
    suspend operator fun invoke(assetId: AssetId, enabled: Boolean)
}
