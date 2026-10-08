package com.gemwallet.android.ui.models.name

import uniffi.gemstone.GemErrorText
import uniffi.gemstone.GemRecipientErrorDisplay

sealed interface AddressInputError {
    data class Rejected(val display: GemRecipientErrorDisplay) : AddressInputError

    data class Failed(val error: GemErrorText) : AddressInputError
}
