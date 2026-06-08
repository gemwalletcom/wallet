package com.gemwallet.android.blockchain.operators.gemstone

import com.gemwallet.android.blockchain.operators.FindPhraseWord
import uniffi.gemstone.GemMnemonic

class GemFindPhraseWord : FindPhraseWord {
    override fun invoke(query: String): List<String> {
        return GemMnemonic().use { mnemonic ->
            mnemonic.suggestWords(query, null)
        }
    }
}
