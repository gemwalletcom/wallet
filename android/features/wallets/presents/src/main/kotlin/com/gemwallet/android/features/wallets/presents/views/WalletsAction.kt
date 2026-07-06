package com.gemwallet.android.features.wallets.presents.views

import com.wallet.core.primitives.WalletId

internal sealed interface WalletsAction {
    data object Create : WalletsAction
    data object Import : WalletsAction
    data class Edit(val walletId: WalletId) : WalletsAction
    data class Select(val walletId: WalletId) : WalletsAction
    data class Delete(val walletId: WalletId) : WalletsAction
    data class TogglePin(val walletId: WalletId) : WalletsAction
    data object Cancel : WalletsAction
}
