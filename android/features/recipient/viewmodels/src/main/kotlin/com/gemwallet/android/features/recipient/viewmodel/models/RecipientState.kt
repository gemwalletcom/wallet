package com.gemwallet.android.features.recipient.viewmodel.models

import com.wallet.core.primitives.Asset
import uniffi.gemstone.GemRecipientType

sealed interface RecipientState {
    data object Loading : RecipientState
    data class Ready(val asset: Asset, val type: GemRecipientType) : RecipientState
}
