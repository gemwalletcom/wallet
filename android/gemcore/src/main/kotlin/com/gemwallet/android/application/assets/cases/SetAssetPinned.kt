package com.gemwallet.android.application.assets.cases

import com.wallet.core.primitives.AssetId

interface SetAssetPinned {
    suspend operator fun invoke(assetId: AssetId, pinned: Boolean)
}
