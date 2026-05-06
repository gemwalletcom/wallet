package com.gemwallet.android.cases.transactions

import com.wallet.core.primitives.Transaction


interface SaveTransactions {
    suspend fun saveTransactions(walletId: String, transactions: List<Transaction>)
}
