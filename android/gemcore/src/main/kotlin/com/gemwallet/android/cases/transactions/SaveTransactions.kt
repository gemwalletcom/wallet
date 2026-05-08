package com.gemwallet.android.cases.transactions

import com.wallet.core.primitives.Transaction
import com.wallet.core.primitives.WalletId


interface SaveTransactions {
    suspend fun saveTransactions(walletId: WalletId, transactions: List<Transaction>)
}
