package com.gemwallet.android.data.coordinators.transaction

import com.gemwallet.android.application.transactions.cases.GetTransaction
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.service.store.database.TransactionsDao
import com.gemwallet.android.model.TransactionExtended
import com.wallet.core.primitives.TransactionId
import kotlinx.coroutines.flow.Flow

class GetTransactionImpl(
    private val sessionRepository: SessionRepository,
    private val transactionsDao: TransactionsDao,
) : GetTransaction {

    override fun invoke(transactionId: TransactionId): Flow<TransactionExtended?> =
        transactionsDao.walletTransaction(sessionRepository, transactionId)
}
