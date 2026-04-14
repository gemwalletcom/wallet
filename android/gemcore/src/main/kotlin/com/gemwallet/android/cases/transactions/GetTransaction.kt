package com.gemwallet.android.cases.transactions

import com.gemwallet.android.model.TransactionExtended
import kotlinx.coroutines.flow.Flow

interface GetTransaction {
    fun getTransaction(transactionId: String): Flow<TransactionExtended?>
}
