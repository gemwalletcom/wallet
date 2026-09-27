package com.gemwallet.android.features.assets.presents.add

internal sealed interface AddAssetAction {
    data object Scan : AddAssetAction
    data object Add : AddAssetAction
    data object SelectChain : AddAssetAction
    data object Cancel : AddAssetAction
}
