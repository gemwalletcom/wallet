package com.gemwallet.android.blockchain.operators.gemstone

import com.gemwallet.android.blockchain.operators.InvalidPhrase
import com.gemwallet.android.blockchain.operators.InvalidWords
import com.gemwallet.android.blockchain.operators.ValidatePhraseOperator
import com.gemwallet.android.ext.words
import uniffi.gemstone.GemMnemonic

class GemValidatePhraseOperator : ValidatePhraseOperator {
    override fun invoke(data: String): Result<Boolean> {
        val words = data.words()
        return GemMnemonic().use { mnemonic ->
            val invalidWords = mnemonic.findInvalidWords(words)
            when {
                invalidWords.isNotEmpty() -> Result.failure(InvalidWords(invalidWords))
                mnemonic.isValid(words) -> Result.success(true)
                else -> Result.failure(InvalidPhrase)
            }
        }
    }
}
