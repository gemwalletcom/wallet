package com.gemwallet.android.features.recipient.presents

import com.gemwallet.android.features.recipient.viewmodel.models.QrScanField
import com.gemwallet.android.model.DestinationAddress
import com.wallet.core.primitives.NameRecord

internal sealed interface RecipientAction {
    data class SetAddress(val address: String, val nameRecord: NameRecord?) : RecipientAction
    data class SetMemo(val memo: String) : RecipientAction
    data class Scan(val field: QrScanField) : RecipientAction
    data object Next : RecipientAction
    data class Select(val destination: DestinationAddress) : RecipientAction
    data object Cancel : RecipientAction
}
