package com.gemwallet.android.features.wallet.presents

import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.WalletId
import uniffi.gemstone.GemWalletSecretKind

internal sealed interface WalletAction {
    data class SetName(val name: String) : WalletAction
    data object SelectImage : WalletAction
    data class ShowPhrase(val walletId: WalletId, val secretKind: GemWalletSecretKind) : WalletAction
    data class ShowPrivateKey(val walletId: WalletId, val chains: List<Chain>) : WalletAction
    data object Delete : WalletAction
    data object Cancel : WalletAction
}
