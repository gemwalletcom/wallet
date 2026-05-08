package com.gemwallet.android.cases.transactions

import com.gemwallet.android.model.TransactionExtended
import com.wallet.core.primitives.TransactionId
import kotlinx.coroutines.flow.Flow

interface GetTransaction {
    fun getTransaction(transactionId: TransactionId): Flow<TransactionExtended?>
}
