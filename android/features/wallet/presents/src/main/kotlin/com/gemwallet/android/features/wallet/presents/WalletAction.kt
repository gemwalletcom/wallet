package com.gemwallet.android.features.wallet.presents

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.NFTAssetId
import uniffi.gemstone.GemBannerKey

sealed interface WalletAction {
    sealed interface Navigation : WalletAction

    data object ShowWallets : Navigation
    data object Manage : Navigation
    data object Search : Navigation
    data object Scan : Navigation
    data object Send : Navigation
    data object Receive : Navigation
    data object Buy : Navigation
    data object Swap : Navigation
    data object Portfolio : Navigation
    data object Perpetuals : Navigation
    data class OpenPerpetual(val assetId: AssetId) : Navigation
    data class OpenAsset(val assetId: AssetId) : Navigation
    data object OpenCollections : Navigation
    data class OpenNftCollection(val collectionId: String) : Navigation
    data class OpenNftAsset(val assetId: NFTAssetId) : Navigation

    data object Refresh : WalletAction
    data object HideBalances : WalletAction
    data class CloseBanner(val key: GemBannerKey) : WalletAction
}
