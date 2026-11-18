package com.gemwallet.android.features.wallet.presents

import com.wallet.core.primitives.WalletId
import uniffi.gemstone.GemWalletRow

sealed interface WalletsAction {
    sealed interface Navigation : WalletsAction

    data class Select(val row: GemWalletRow) : WalletsAction
    data class Delete(val row: GemWalletRow) : WalletsAction
    data class TogglePin(val row: GemWalletRow) : WalletsAction

    data object Create : Navigation
    data object Import : Navigation
    data class Edit(val walletId: WalletId) : Navigation
    data object Cancel : Navigation
}
