package com.gemwallet.android.features.assets.views

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetTag

sealed interface WalletSearchAction {
    data object AddAsset : WalletSearchAction
    data object Cancel : WalletSearchAction
    data object OpenPerpetuals : WalletSearchAction
    data class OpenAsset(val assetId: AssetId) : WalletSearchAction
    data class OpenPerpetual(val assetId: AssetId) : WalletSearchAction
    data class ShowAllAssets(val query: String, val tag: AssetTag?) : WalletSearchAction
}
