package com.gemwallet.android.features.assets.views

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.NFTAssetId

sealed interface WalletAction {
    data object ShowWallets : WalletAction
    data object Search : WalletAction
    data object Manage : WalletAction
    data object Send : WalletAction
    data object Receive : WalletAction
    data object Buy : WalletAction
    data object Swap : WalletAction
    data object Perpetuals : WalletAction
    data class OpenPerpetualDetails(val assetId: AssetId) : WalletAction
    data class OpenAsset(val assetId: AssetId) : WalletAction
    data class OpenNftCollection(val collectionId: String) : WalletAction
    data class OpenNftAsset(val assetId: NFTAssetId) : WalletAction
    data object NftReceive : WalletAction
    data object NftUnverified : WalletAction
}
