package com.gemwallet.android.model

import com.wallet.core.primitives.AssetId

sealed interface FeeAssetSelection {
    data object Automatic : FeeAssetSelection
    data class Selected(val assetId: AssetId) : FeeAssetSelection
}
