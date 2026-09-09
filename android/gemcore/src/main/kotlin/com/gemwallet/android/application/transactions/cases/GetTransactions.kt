package com.gemwallet.android.application.transactions.cases

import com.gemwallet.android.domains.transaction.aggregates.TransactionDataAggregate
import kotlinx.coroutines.flow.Flow

interface GetTransactions {
    fun getTransactions(
        filters: List<TransactionsRequestFilter> = emptyList(),
    ): Flow<List<TransactionDataAggregate>>
}
