package com.gemwallet.android.features.add_asset.views

internal sealed interface AddAssetAction {
    data object Scan : AddAssetAction
    data object Add : AddAssetAction
    data object SelectChain : AddAssetAction
    data object Cancel : AddAssetAction
}
