package com.gemwallet.android.data.coordinators.transaction

import com.gemwallet.android.application.transactions.cases.GetPendingTransactionsCount
import com.gemwallet.android.application.transactions.cases.TransactionsRequestFilter
import com.gemwallet.android.cases.transactions.ClearPendingTransactions
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.service.store.database.TransactionsDao
import com.wallet.core.primitives.TransactionState
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flatMapLatest

private val pendingTransactionStates = listOf(TransactionState.Pending, TransactionState.InTransit)

@OptIn(ExperimentalCoroutinesApi::class)
class GetPendingTransactionsCountImpl(
    private val sessionRepository: SessionRepository,
    private val transactionsDao: TransactionsDao,
) : GetPendingTransactionsCount {

    override fun getPendingTransactionsCount(): Flow<Int?> = sessionRepository.currentWalletId()
        .flatMapLatest { walletId ->
            transactionsDao.getTransactionsCount(
                walletId,
                TransactionsRequestFilter.activityDefaults() + TransactionsRequestFilter.States(pendingTransactionStates),
            )
        }
}

class ClearPendingTransactionsImpl(
    private val transactionsDao: TransactionsDao,
) : ClearPendingTransactions {

    override suspend fun clearPending() {
        transactionsDao.deleteByState(TransactionState.Pending)
    }
}
