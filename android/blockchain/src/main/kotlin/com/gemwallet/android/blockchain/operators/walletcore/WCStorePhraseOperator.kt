package com.gemwallet.android.blockchain.operators.walletcore

import com.gemwallet.android.blockchain.operators.StorePhraseOperator
import com.gemwallet.android.math.decodeHex
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletType
import wallet.core.jni.CoinType
import wallet.core.jni.StoredKey

class WCStorePhraseOperator(
    private val keyStoreDir: String,
    private val coinTypeProxy: WCChainTypeProxy = WCChainTypeProxy()
) : StorePhraseOperator {
    override suspend fun invoke(wallet: Wallet, data: String, password: String): Result<Boolean> = try {
        val walletId = wallet.id.id
        val storedKey = if (wallet.type == WalletType.PrivateKey) {
            val coinType = coinTypeProxy(wallet.accounts.firstOrNull()?.chain ?: throw IllegalArgumentException())
            StoredKey.importPrivateKey(
                data.decodeHex(),
                walletId,
                password.decodeHex(),
                coinType,
            )
        } else {
            StoredKey.importHDWallet(data, walletId, password.decodeHex(), CoinType.BITCOIN)
        }
        storedKey.store("$keyStoreDir/$walletId")
        Result.success(true)
    } catch (err: Throwable) {
        Result.failure(err)
    }
}
