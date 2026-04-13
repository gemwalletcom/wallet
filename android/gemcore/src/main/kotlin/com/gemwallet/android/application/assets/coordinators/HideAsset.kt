package com.gemwallet.android.application.assets.coordinators

import com.wallet.core.primitives.AssetId

interface HideAsset {
    suspend operator fun invoke(assetId: AssetId)
}
