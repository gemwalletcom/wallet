package com.gemwallet.android.features.assets.views

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.NFTAssetId
import com.wallet.core.primitives.PerpetualId

sealed interface WalletSearchAction {
    data object AddAsset : WalletSearchAction
    data object Cancel : WalletSearchAction
    data object OpenPerpetuals : WalletSearchAction
    data object OpenCollections : WalletSearchAction
    data object OpenRecentsSheet : WalletSearchAction
    data class OpenAsset(val assetId: AssetId) : WalletSearchAction
    data class OpenPerpetual(val assetId: AssetId) : WalletSearchAction
    data class OpenNftCollection(val collectionId: String) : WalletSearchAction
    data class OpenNftAsset(val assetId: NFTAssetId) : WalletSearchAction
    data class OpenRecent(val assetId: AssetId) : WalletSearchAction
    data class ShowAllAssets(val query: String) : WalletSearchAction
    data class OpenList(val listId: String, val title: String) : WalletSearchAction
    data class PinAsset(val assetId: AssetId) : WalletSearchAction
    data class AddToWallet(val assetId: AssetId) : WalletSearchAction
    data class TogglePerpetualPin(val perpetualId: PerpetualId) : WalletSearchAction
}
