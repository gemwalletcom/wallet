package com.gemwallet.android.blockchain.operators.gemstone

import com.gemwallet.android.blockchain.operators.CreateWalletOperator
import uniffi.gemstone.GemMnemonic

class GemCreateWalletOperator : CreateWalletOperator {
    override suspend fun invoke(): Result<String> = runCatching {
        GemMnemonic().use { mnemonic ->
            mnemonic.generate(wordCount = 12u).joinToString(" ")
        }
    }
}
