package com.gemwallet.android.application.assets.coordinators

import com.wallet.core.primitives.AssetId

interface ToggleAssetPin {
    suspend operator fun invoke(assetId: AssetId)
}
