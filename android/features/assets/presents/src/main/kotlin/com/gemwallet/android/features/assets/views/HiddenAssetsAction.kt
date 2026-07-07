package com.gemwallet.android.features.assets.views

import com.wallet.core.primitives.AssetId

sealed interface HiddenAssetsAction {
    data object Close : HiddenAssetsAction
    data class OpenAsset(val assetId: AssetId) : HiddenAssetsAction
}
