package com.gemwallet.android.blockchain.services

import com.gemwallet.android.blockchain.gemstone.toGemSignerInput
import com.gemwallet.android.blockchain.operators.gemstone.withGemKeystore
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.ext.keystoreId
import com.gemwallet.android.model.SignerParams
import com.wallet.core.primitives.Wallet

class GemSignTransactionOperator(
    private val baseDir: String,
) {
    suspend operator fun invoke(wallet: Wallet, params: SignerParams, password: String): List<ByteArray> {
        val keystoreId = wallet.keystoreId
        val chain = params.input.asset.chain.string
        val data = params.data()
        val gemInput = params.input.toGemSignerInput(data.metadata, params.finalAmount, data.fee)
        return withGemKeystore(baseDir, password) { keystore, passwordBytes ->
            keystore.sign(keystoreId, chain, gemInput, passwordBytes).map { it.toByteArray() }
        }
    }
}
