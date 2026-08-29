package com.gemwallet.android.data.coordinators.transaction

import com.gemwallet.android.application.transactions.cases.GetTransaction
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.gemstone.GemstoneTransactionStore
import com.gemwallet.android.model.TransactionExtended
import com.wallet.core.primitives.TransactionId
import kotlinx.coroutines.flow.Flow

class GetTransactionImpl(
    private val sessionRepository: SessionRepository,
    private val transactionStore: GemstoneTransactionStore,
) : GetTransaction {

    override fun invoke(transactionId: TransactionId): Flow<TransactionExtended?> =
        transactionStore.walletTransaction(sessionRepository, transactionId)
}
