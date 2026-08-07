package com.gemwallet.android.blockchain.operators.gemstone

import uniffi.gemstone.GemMnemonic

class GemFindPhraseWord {
    operator fun invoke(query: String): List<String> {
        return GemMnemonic().use { mnemonic ->
            mnemonic.suggestWords(query, null)
        }
    }
}
