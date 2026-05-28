package com.gemwallet.android.blockchain.operators.walletcore

import com.gemwallet.android.blockchain.operators.LoadPrivateDataOperator
import com.gemwallet.android.math.fromHex
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletType
import wallet.core.jni.StoredKey

class WCLoadPrivateDataOperator(
    private val keyStoreDir: String
) : LoadPrivateDataOperator {
    override suspend fun invoke(wallet: Wallet, password: String): String {
        val walletId = wallet.id.id
        val storeKey = StoredKey.load("$keyStoreDir/$walletId")
            ?: throw IllegalStateException("Failed to load stored key for wallet $walletId")

        return if (wallet.type == WalletType.PrivateKey) {
            val chain = wallet.accounts.firstOrNull()?.chain
                ?: throw IllegalStateException("No accounts found for wallet $walletId")
            val bytes = storeKey.decryptPrivateKey(password.fromHex())
            uniffi.gemstone.encodePrivateKey(chain.string, bytes)
        } else {
            storeKey.decryptMnemonic(password.fromHex())
        }
    }

}
