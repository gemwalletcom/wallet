package com.gemwallet.android.features.wallet.presents

import uniffi.gemstone.GemWalletRow

internal sealed interface WalletsAction {
    data object Create : WalletsAction
    data object Import : WalletsAction
    data class Edit(val row: GemWalletRow) : WalletsAction
    data class Select(val row: GemWalletRow) : WalletsAction
    data class Delete(val row: GemWalletRow) : WalletsAction
    data class TogglePin(val row: GemWalletRow) : WalletsAction
    data object Cancel : WalletsAction
}
