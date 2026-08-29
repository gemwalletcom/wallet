package com.gemwallet.android.application.transactions.cases

import com.gemwallet.android.model.TransactionExtended
import com.wallet.core.primitives.TransactionId
import kotlinx.coroutines.flow.Flow

interface GetTransaction {
    operator fun invoke(transactionId: TransactionId): Flow<TransactionExtended?>
}
