package com.gemwallet.android.blockchain.operators

import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Wallet

interface AddAccountsOperator {
    suspend operator fun invoke(wallet: Wallet, chains: List<Chain>, password: String): List<Account>
}
