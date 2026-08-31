package com.gemwallet.android.application.assets.cases

import com.wallet.core.primitives.AssetId

interface HideAsset {
    suspend operator fun invoke(assetId: AssetId)
}
