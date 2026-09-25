package com.gemwallet.android.application.transactions.cases

import kotlinx.coroutines.flow.Flow
import uniffi.gemstone.GemTransactionRow

interface GetTransactions {
    fun getTransactions(filters: List<TransactionsRequestFilter> = emptyList()): Flow<List<GemTransactionRow>>

    fun stored(filters: List<TransactionsRequestFilter> = emptyList()): List<GemTransactionRow>
}
