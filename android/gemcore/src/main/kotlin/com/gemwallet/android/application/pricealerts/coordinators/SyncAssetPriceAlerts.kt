package com.gemwallet.android.application.pricealerts.coordinators

import com.wallet.core.primitives.AssetId

interface SyncAssetPriceAlerts {
    suspend operator fun invoke(assetId: AssetId)
}
