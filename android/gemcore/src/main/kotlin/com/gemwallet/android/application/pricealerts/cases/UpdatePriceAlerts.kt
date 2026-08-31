package com.gemwallet.android.application.pricealerts.cases

import com.wallet.core.primitives.AssetId

interface UpdatePriceAlerts {
    suspend fun update()
    suspend fun update(assetId: AssetId)
}
