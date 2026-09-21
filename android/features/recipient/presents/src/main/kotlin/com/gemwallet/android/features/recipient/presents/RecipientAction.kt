package com.gemwallet.android.features.recipient.presents

import com.gemwallet.android.features.recipient.viewmodel.models.QrScanField
import uniffi.gemstone.GemRecipient

internal sealed interface RecipientAction {
    data class SetAddress(val address: String) : RecipientAction
    data class SetMemo(val memo: String) : RecipientAction
    data class Scan(val field: QrScanField) : RecipientAction
    data object Next : RecipientAction
    data object ValidateAddress : RecipientAction
    data class Select(val destination: GemRecipient) : RecipientAction
    data object Cancel : RecipientAction
}
