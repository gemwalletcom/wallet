package com.gemwallet.android.features.wallet.presents

import com.gemwallet.android.domains.wallet.WalletSecretInput

internal sealed interface WalletDetailAction {
    data class SetName(val name: String) : WalletDetailAction
    data object SelectImage : WalletDetailAction
    data class ShowPhrase(val input: WalletSecretInput) : WalletDetailAction
    data object Delete : WalletDetailAction
    data object Cancel : WalletDetailAction
}
