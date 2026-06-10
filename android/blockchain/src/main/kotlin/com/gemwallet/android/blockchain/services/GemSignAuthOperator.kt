package com.gemwallet.android.blockchain.services

import com.gemwallet.android.blockchain.operators.gemstone.withGemKeystore
import com.gemwallet.android.ext.keystoreId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Wallet

class GemSignAuthOperator(
    private val baseDir: String,
) {
    suspend operator fun invoke(wallet: Wallet, chain: Chain, hash: ByteArray, password: String): String {
        val keystoreId = wallet.keystoreId
        return withGemKeystore(baseDir, password) { keystore, passwordBytes ->
            keystore.signAuth(keystoreId, chain.string, hash, passwordBytes)
        }
    }
}
