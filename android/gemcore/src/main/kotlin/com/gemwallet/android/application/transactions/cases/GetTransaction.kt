package com.gemwallet.android.application.transactions.cases

import com.wallet.core.primitives.TransactionExtended
import com.wallet.core.primitives.TransactionId
import kotlinx.coroutines.flow.Flow

interface GetTransaction {
    operator fun invoke(transactionId: TransactionId): Flow<TransactionExtended?>
}
