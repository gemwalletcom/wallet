package com.gemwallet.android.blockchain.services

import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.application.getKeystorePassword
import com.gemwallet.android.blockchain.operators.gemstone.withGemKeystore
import com.wallet.core.primitives.Wallet
import uniffi.gemstone.MessageSigner

class GemSignMessageOperator(
    private val baseDir: String,
    private val passwordStore: PasswordStore,
) {
    suspend fun sign(signer: MessageSigner, wallet: Wallet): String {
        return withGemKeystore(baseDir, passwordStore.getKeystorePassword()) { keystore, passwordBytes ->
            signer.signWithKeystore(keystore, keystore.keystoreId(wallet.id.id), passwordBytes)
        }
    }
}
