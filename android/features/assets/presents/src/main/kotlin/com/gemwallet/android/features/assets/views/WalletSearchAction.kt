package com.gemwallet.android.features.assets.views

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetTag
import com.wallet.core.primitives.PerpetualId

sealed interface WalletSearchAction {
    data object AddAsset : WalletSearchAction
    data object Cancel : WalletSearchAction
    data object OpenPerpetuals : WalletSearchAction
    data object OpenRecentsSheet : WalletSearchAction
    data class OpenAsset(val assetId: AssetId) : WalletSearchAction
    data class OpenPerpetual(val assetId: AssetId) : WalletSearchAction
    data class OpenRecent(val assetId: AssetId) : WalletSearchAction
    data class ShowAllAssets(val query: String, val tag: AssetTag?) : WalletSearchAction
    data class SelectTag(val tag: AssetTag?) : WalletSearchAction
    data class PinAsset(val assetId: AssetId) : WalletSearchAction
    data class AddToWallet(val assetId: AssetId) : WalletSearchAction
    data class TogglePerpetualPin(val perpetualId: PerpetualId) : WalletSearchAction
}
