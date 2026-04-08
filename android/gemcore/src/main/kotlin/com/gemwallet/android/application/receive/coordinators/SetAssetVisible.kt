package com.gemwallet.android.application.receive.coordinators

import com.wallet.core.primitives.AssetId

interface SetAssetVisible {
    suspend operator fun invoke(assetId: AssetId)
}
