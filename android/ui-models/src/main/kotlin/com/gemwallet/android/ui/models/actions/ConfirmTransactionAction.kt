package com.gemwallet.android.ui.models.actions

import uniffi.gemstone.GemConfirmInput

fun interface ConfirmTransactionAction {
    operator fun invoke(input: GemConfirmInput)
}
