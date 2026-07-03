package com.gemwallet.android.features.wallet.presents

import com.wallet.core.primitives.WalletId
import com.wallet.core.primitives.WalletType

internal sealed interface WalletAction {
    data class SetName(val name: String) : WalletAction
    data object SelectImage : WalletAction
    data class ShowPhrase(val walletId: WalletId, val walletType: WalletType) : WalletAction
    data object Delete : WalletAction
    data object Cancel : WalletAction
}
