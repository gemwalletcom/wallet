package com.gemwallet.android.features.wallet.presents

import uniffi.gemstone.GemSecretExport

internal sealed interface WalletDetailAction {
    data class SetName(val name: String) : WalletDetailAction
    data object SelectImage : WalletDetailAction
    data class ShowSecret(val export: GemSecretExport) : WalletDetailAction
    data object Delete : WalletDetailAction
    data object Cancel : WalletDetailAction
}
