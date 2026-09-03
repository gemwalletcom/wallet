package com.gemwallet.android.ui.models.actions

import uniffi.gemstone.GemTransferData

fun interface ConfirmTransactionAction {
    operator fun invoke(transfer: GemTransferData)
}
