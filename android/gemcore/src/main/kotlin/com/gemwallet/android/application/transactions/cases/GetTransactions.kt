package com.gemwallet.android.application.transactions.cases

import com.gemwallet.android.application.transactions.values.TransactionsQueryFilter
import kotlinx.coroutines.flow.Flow
import uniffi.gemstone.GemTransactionRow

interface GetTransactions {
    fun getTransactions(filters: List<TransactionsQueryFilter> = emptyList()): Flow<List<GemTransactionRow>>

    fun stored(filters: List<TransactionsQueryFilter> = emptyList()): List<GemTransactionRow>
}
