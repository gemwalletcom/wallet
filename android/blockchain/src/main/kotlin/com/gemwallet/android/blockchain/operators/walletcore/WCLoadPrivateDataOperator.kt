package com.gemwallet.android.blockchain.operators.walletcore

import com.gemwallet.android.blockchain.operators.LoadPrivateDataOperator
import com.gemwallet.android.math.decodeHex
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletType
import wallet.core.jni.StoredKey

class WCLoadPrivateDataOperator(
    private val keyStoreDir: String
) : LoadPrivateDataOperator {
    override suspend fun invoke(wallet: Wallet, password: String): String {
        val storeKey = StoredKey.load("$keyStoreDir/${wallet.id}")
            ?: throw IllegalStateException("Failed to load stored key for wallet ${wallet.id}")

        return if (wallet.type == WalletType.PrivateKey) {
            val chain = wallet.accounts.firstOrNull()?.chain
                ?: throw IllegalStateException("No accounts found for wallet ${wallet.id}")
            val bytes = storeKey.decryptPrivateKey(password.decodeHex())
            uniffi.gemstone.encodePrivateKey(chain.string, bytes)
        } else {
            storeKey.decryptMnemonic(password.decodeHex())
        }
    }

}