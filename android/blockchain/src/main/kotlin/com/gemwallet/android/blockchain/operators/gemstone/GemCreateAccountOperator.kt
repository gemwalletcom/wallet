package com.gemwallet.android.blockchain.operators.gemstone

import com.gemwallet.android.blockchain.operators.CreateAccountOperator
import com.gemwallet.android.ext.words
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.WalletType
import uniffi.gemstone.GemImportType
import uniffi.gemstone.GemKeystore

class GemCreateAccountOperator(
    private val baseDir: String,
) : CreateAccountOperator {

    override fun invoke(walletType: WalletType, data: String, chain: Chain): Account {
        val walletImport = when (walletType) {
            WalletType.Multicoin -> GemImportType.MulticoinPhrase(words = data.words(), chains = listOf(chain.string))
            WalletType.Single -> GemImportType.SinglePhrase(words = data.words(), chain = chain.string)
            WalletType.PrivateKey -> GemImportType.PrivateKey(value = data, chain = chain.string)
            WalletType.View -> throw IllegalArgumentException("View wallets do not derive accounts")
        }
        return GemKeystore(baseDir).use { keystore ->
            keystore.previewImport(walletImport).accounts.first().toAccount()
        }
    }
}
