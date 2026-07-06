package com.gemwallet.android.features.assets.views

import com.wallet.core.primitives.AssetId

sealed interface AssetsAction {
    data object ShowWallets : AssetsAction
    data object Manage : AssetsAction
    data object Search : AssetsAction
    data object Send : AssetsAction
    data object Receive : AssetsAction
    data object Buy : AssetsAction
    data object Swap : AssetsAction
    data object Perpetuals : AssetsAction
    data class OpenPerpetualDetails(val assetId: AssetId) : AssetsAction
    data class OpenAsset(val assetId: AssetId) : AssetsAction
}
