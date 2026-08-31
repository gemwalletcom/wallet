package com.gemwallet.android.features.recipient.presents

import uniffi.gemstone.GemRecipient
import com.gemwallet.android.features.recipient.viewmodel.models.QrScanField

internal sealed interface RecipientAction {
    data class SetAddress(val address: String) : RecipientAction
    data class SetMemo(val memo: String) : RecipientAction
    data class Scan(val field: QrScanField) : RecipientAction
    data object Next : RecipientAction
    data class Select(val destination: GemRecipient) : RecipientAction
    data object Cancel : RecipientAction
}
