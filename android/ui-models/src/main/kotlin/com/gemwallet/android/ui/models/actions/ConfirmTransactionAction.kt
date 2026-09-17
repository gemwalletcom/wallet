package com.gemwallet.android.ui.models.actions

import com.gemwallet.android.domains.confirm.ConfirmTransferInput

fun interface ConfirmTransactionAction {
    operator fun invoke(input: ConfirmTransferInput)
}
