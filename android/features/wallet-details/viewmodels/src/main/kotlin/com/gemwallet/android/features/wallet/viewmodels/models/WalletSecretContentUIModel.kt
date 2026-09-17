package com.gemwallet.android.features.wallet.viewmodels.models

import uniffi.gemstone.GemWalletSecret

sealed interface WalletSecretContentUIModel {
    val text: String

    data class PrivateKey(val key: String) : WalletSecretContentUIModel {
        override val text: String get() = key
    }

    data class Words(val words: List<String>) : WalletSecretContentUIModel {
        override val text: String get() = words.joinToString(" ")
    }
}

internal fun GemWalletSecret.uiModel(): WalletSecretContentUIModel = when (this) {
    is GemWalletSecret.PrivateKey -> WalletSecretContentUIModel.PrivateKey(key)
    is GemWalletSecret.Words -> WalletSecretContentUIModel.Words(words)
}
