package com.gemwallet.android.blockchain.operators.gemstone

import com.gemwallet.android.blockchain.operators.LoadPrivateDataOperator
import com.gemwallet.android.ext.keystoreId
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletType

class GemLoadPrivateDataOperator(
    private val baseDir: String,
) : LoadPrivateDataOperator {

    override suspend fun invoke(wallet: Wallet, password: String): String =
        withGemKeystore(baseDir, password) { keystore, passwordBytes ->
            if (wallet.type == WalletType.PrivateKey) {
                val chain = wallet.accounts.firstOrNull()?.chain
                    ?: throw IllegalStateException("No accounts found for wallet ${wallet.id.id}")
                keystore.exportPrivateKey(wallet.keystoreId, chain.string, passwordBytes)
            } else {
                keystore.exportRecoveryPhrase(wallet.keystoreId, passwordBytes).joinToString(" ")
            }
        }
}
