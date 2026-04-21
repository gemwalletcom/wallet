package com.gemwallet.android.blockchain.operators.walletcore

import com.gemwallet.android.blockchain.operators.LoadPublicKeyOperator
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Wallet
import wallet.core.jni.StoredKey

class WCLoadPublicKeyOperator(
    private val keyStoreDir: String,
) : LoadPublicKeyOperator {
    override fun invoke(wallet: Wallet, chain: Chain): String? {
        val coinType = WCChainTypeProxy().invoke(chain = chain)
        return try {
            val store = StoredKey.load("$keyStoreDir/${wallet.id}")
            (0 until store.accountCount())
                .mapNotNull { store.account(it) }
                .firstOrNull { it.coin() == coinType }
                ?.publicKey()
                ?.takeIf { it.isNotEmpty() }
        } catch (_: Throwable) {
            null
        }
    }
}
