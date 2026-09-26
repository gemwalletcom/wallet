package com.gemwallet.android.application.transactions.cases

import com.wallet.core.primitives.TransactionsFilter
import kotlinx.coroutines.flow.Flow
import uniffi.gemstone.GemTransactionRow

interface GetTransactions {
    fun getTransactions(filter: TransactionsFilter?, limit: Int): Flow<List<GemTransactionRow>>

    fun stored(filter: TransactionsFilter?, limit: Int): List<GemTransactionRow>
}
