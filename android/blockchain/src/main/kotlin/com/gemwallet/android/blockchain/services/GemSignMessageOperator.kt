package com.gemwallet.android.blockchain.services

import com.gemwallet.android.blockchain.operators.gemstone.withGemKeystore
import com.wallet.core.primitives.Wallet
import uniffi.gemstone.MessageSigner

class GemSignMessageOperator(
    private val baseDir: String,
) {
    suspend fun sign(signer: MessageSigner, wallet: Wallet, password: String): String {
        return withGemKeystore(baseDir, password) { keystore, passwordBytes ->
            signer.signWithKeystore(keystore, keystore.keystoreId(wallet.id.id), passwordBytes)
        }
    }
}
