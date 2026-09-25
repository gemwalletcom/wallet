package com.gemwallet.android.application.transactions.cases

import com.wallet.core.primitives.TransactionId
import kotlinx.coroutines.flow.Flow
import uniffi.gemstone.GemTransactionDetailRows

data class TransactionDetails(val rows: GemTransactionDetailRows)

interface GetTransactionDetails {
    fun getTransactionDetails(id: TransactionId): Flow<TransactionDetails?>
}
