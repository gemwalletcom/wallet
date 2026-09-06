package com.gemwallet.android.blockchain.services

import android.util.Log
import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.application.getKeystorePassword
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.blockchain.operators.gemstone.withGemKeystore
import kotlinx.coroutines.CancellationException
import uniffi.gemstone.GemSignedTransaction
import uniffi.gemstone.GemSignerInput
import uniffi.gemstone.GemTransactionSigner

class KeystoreTransactionSigner(
    private val baseDir: String,
    private val passwordStore: PasswordStore,
) : GemTransactionSigner {
    override suspend fun sign(wallet: uniffi.gemstone.Wallet, input: GemSignerInput): List<GemSignedTransaction> {
        return try {
            val password = passwordStore.getKeystorePassword()
            val chain = input.input.chain()
            withGemKeystore(baseDir, password) { keystore, passwordBytes ->
                keystore.sign(keystore.keystoreId(wallet.id), chain, input, passwordBytes)
            }
        } catch (error: CancellationException) {
            throw error
        } catch (error: Exception) {
            Log.e(TAG, "keystore transaction signing failed (${error.javaClass.simpleName})")
            throw error
        }
    }

    private companion object {
        const val TAG = "KeystoreTransactionSigner"
    }
}
