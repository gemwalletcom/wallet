package com.gemwallet.android.data.repositories.transactions

import com.gemwallet.android.application.transactions.coordinators.TransactionsRequestFilter
import com.gemwallet.android.model.TransactionExtended
import kotlinx.coroutines.flow.Flow

interface TransactionRepository {
    fun getTransactions(filters: List<TransactionsRequestFilter> = emptyList()): Flow<List<TransactionExtended>>

    fun getTransaction(transactionId: String): Flow<TransactionExtended?>
}
