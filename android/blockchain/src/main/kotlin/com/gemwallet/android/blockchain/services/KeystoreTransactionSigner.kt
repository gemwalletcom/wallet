package com.gemwallet.android.blockchain.services

import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.blockchain.operators.gemstone.withGemKeystore
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.Wallet
import uniffi.gemstone.GemSignedTransaction
import uniffi.gemstone.GemSignerInput
import uniffi.gemstone.GemTransactionSigner
import uniffi.gemstone.GemTransferService

class KeystoreTransactionSigner(
    private val baseDir: String,
    private val passwordStore: PasswordStore,
    private val transferService: GemTransferService,
) : GemTransactionSigner {
    override suspend fun sign(wallet: String, input: GemSignerInput): List<GemSignedTransaction> {
        val wallet = wallet.decodeJson<Wallet>()
        val chain = transferService.asset(input.input.inputType).toPrimitives().id.chain.string
        return withGemKeystore(baseDir, passwordStore.getPassword(wallet.id.id)) { keystore, passwordBytes ->
            keystore.sign(keystore.keystoreId(wallet.id.id), chain, input, passwordBytes)
        }
    }
}
