package com.gemwallet.android.application.receive.cases

import com.wallet.core.primitives.AssetId

interface SetAssetVisible {
    suspend operator fun invoke(assetId: AssetId)
}
