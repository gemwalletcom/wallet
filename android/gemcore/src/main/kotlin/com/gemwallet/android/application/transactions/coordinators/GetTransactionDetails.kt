package com.gemwallet.android.application.transactions.coordinators

import com.gemwallet.android.domains.transaction.aggregates.TransactionDetailsAggregate
import com.wallet.core.primitives.TransactionId
import kotlinx.coroutines.flow.Flow

interface GetTransactionDetails {
    fun getTransactionDetails(id: TransactionId): Flow<TransactionDetailsAggregate?>
}