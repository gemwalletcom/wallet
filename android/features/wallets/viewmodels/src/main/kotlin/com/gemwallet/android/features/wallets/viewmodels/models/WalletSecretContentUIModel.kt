package com.gemwallet.android.features.wallets.viewmodels.models

import com.gemwallet.android.ui.components.screen.PhraseRow
import com.gemwallet.android.ui.components.screen.phraseRows
import uniffi.gemstone.GemCopy
import uniffi.gemstone.GemWalletSecret
import uniffi.gemstone.privateKeyCopy
import uniffi.gemstone.secretPhraseCopy

sealed interface WalletSecretContentUIModel {
    fun copy(): GemCopy

    class PrivateKey(val key: String) : WalletSecretContentUIModel {
        override fun copy(): GemCopy = privateKeyCopy(key)
    }

    class Words(private val words: List<String>) : WalletSecretContentUIModel {
        val rows: List<PhraseRow> = phraseRows(words)

        override fun copy(): GemCopy = secretPhraseCopy(words)
    }
}

internal fun GemWalletSecret.uiModel(): WalletSecretContentUIModel = when (this) {
    is GemWalletSecret.PrivateKey -> WalletSecretContentUIModel.PrivateKey(key)
    is GemWalletSecret.Words -> WalletSecretContentUIModel.Words(words)
}
