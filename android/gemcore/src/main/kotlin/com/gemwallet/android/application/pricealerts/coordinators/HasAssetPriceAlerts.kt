package com.gemwallet.android.application.pricealerts.coordinators

import com.wallet.core.primitives.AssetId

interface HasAssetPriceAlerts {
    suspend operator fun invoke(assetId: AssetId): Boolean
}
