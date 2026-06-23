package com.gemwallet.android.features.recipient.viewmodel.models

sealed interface RecipientError {
    data object None : RecipientError
    data class IncorrectAddress(val assetName: String) : RecipientError
}
