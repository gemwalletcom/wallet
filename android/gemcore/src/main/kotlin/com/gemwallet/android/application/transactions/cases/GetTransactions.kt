package com.gemwallet.android.application.transactions.cases

import com.gemwallet.android.domains.transaction.aggregates.TransactionDataAggregate
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.StateFlow

interface GetTransactions {
    fun getTransactions(
        filters: List<TransactionsRequestFilter> = emptyList(),
    ): Flow<List<TransactionDataAggregate>>

    fun transactions(): StateFlow<List<TransactionDataAggregate>>
}
