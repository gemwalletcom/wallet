package com.gemwallet.android.application.receive.coordinators

import com.wallet.core.primitives.AssetId

interface SetAssetVisible {
    suspend fun setAssetVisible(assetId: AssetId)
}
