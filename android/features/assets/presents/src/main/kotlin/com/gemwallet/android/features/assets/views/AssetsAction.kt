package com.gemwallet.android.features.assets.views

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.NFTAssetId

sealed interface AssetsAction {
    data object ShowWallets : AssetsAction
    data object Manage : AssetsAction
    data object Search : AssetsAction
    data object Scan : AssetsAction
    data object Send : AssetsAction
    data object Receive : AssetsAction
    data object Buy : AssetsAction
    data object Swap : AssetsAction
    data object Portfolio : AssetsAction
    data object Perpetuals : AssetsAction
    data class OpenPerpetualDetails(val assetId: AssetId) : AssetsAction
    data class OpenAsset(val assetId: AssetId) : AssetsAction
    data object OpenCollections : AssetsAction
    data class OpenNftCollection(val collectionId: String) : AssetsAction
    data class OpenNftAsset(val assetId: NFTAssetId) : AssetsAction
}
