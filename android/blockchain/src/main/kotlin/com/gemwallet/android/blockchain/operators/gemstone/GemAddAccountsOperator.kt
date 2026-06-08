package com.gemwallet.android.blockchain.operators.gemstone

import com.gemwallet.android.blockchain.operators.AddAccountsOperator
import com.gemwallet.android.ext.keystoreId
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Wallet

class GemAddAccountsOperator(
    private val baseDir: String,
) : AddAccountsOperator {

    override suspend fun invoke(wallet: Wallet, chains: List<Chain>, password: String): List<Account> =
        withGemKeystore(baseDir, password) { keystore, passwordBytes ->
            keystore.addAccounts(wallet.keystoreId, passwordBytes, chains.map { it.string })
                .map { it.toAccount() }
        }
}
