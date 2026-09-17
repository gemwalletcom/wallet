package com.gemwallet.android.features.wallet.presents

import com.gemwallet.android.domains.wallet.WalletSecretInput

internal sealed interface WalletAction {
    data class SetName(val name: String) : WalletAction
    data object SelectImage : WalletAction
    data class ShowPhrase(val input: WalletSecretInput) : WalletAction
    data object Delete : WalletAction
    data object Cancel : WalletAction
}
