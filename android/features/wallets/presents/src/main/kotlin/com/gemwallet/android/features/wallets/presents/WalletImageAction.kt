package com.gemwallet.android.features.wallets.presents

internal sealed interface WalletImageAction {
    data class SetEmoji(val emoji: String, val backgroundColor: Int) : WalletImageAction
    data class SetNftImage(val url: String) : WalletImageAction
    data object ResetToDefault : WalletImageAction
    data object Close : WalletImageAction
}
