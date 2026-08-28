package com.gemwallet.android.application.pricealerts.cases

import com.wallet.core.primitives.AssetId

interface SetAssetPriceAlertEnabled {
    suspend operator fun invoke(assetId: AssetId, enabled: Boolean)
}
