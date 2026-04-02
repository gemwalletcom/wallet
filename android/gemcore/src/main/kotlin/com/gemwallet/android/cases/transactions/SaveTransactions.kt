package com.gemwallet.android.cases.transactions

import com.gemwallet.android.model.Transaction


interface SaveTransactions {
    suspend fun saveTransactions(walletId: String, transactions: List<Transaction>)
}
