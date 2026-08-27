package com.gemwallet.android.data.repositories.transactions

import com.gemwallet.android.application.transactions.coordinators.GetPendingTransactionsCount
import com.gemwallet.android.application.transactions.coordinators.TransactionsRequestFilter
import com.gemwallet.android.cases.transactions.ClearPendingTransactions
import com.gemwallet.android.cases.transactions.CreateTransaction
import com.gemwallet.android.cases.transactions.SaveTransactions
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.service.store.database.TransactionsDao
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.model.TransactionExtended
import com.wallet.core.primitives.Transaction
import com.wallet.core.primitives.TransactionId
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapNotNull
import kotlinx.coroutines.withContext

private val pendingTransactionStates = listOf(TransactionState.Pending, TransactionState.InTransit)

@OptIn(ExperimentalCoroutinesApi::class)
class TransactionsRepositoryImpl(
    private val sessionRepository: SessionRepository,
    private val transactionsDao: TransactionsDao,
) : TransactionRepository,
    GetPendingTransactionsCount,
    CreateTransaction,
    SaveTransactions,
    ClearPendingTransactions {

    private fun currentWalletId(): Flow<WalletId> = sessionRepository.session()
        .filterNotNull()
        .map { it.wallet.id }
        .distinctUntilChanged()

    override fun getPendingTransactionsCount(): Flow<Int?> {
        return currentWalletId().flatMapLatest { walletId ->
            transactionsDao.getTransactionsCount(
                walletId,
                TransactionsRequestFilter.activityDefaults() + TransactionsRequestFilter.States(pendingTransactionStates),
            )
        }
    }

    override fun getTransactions(filters: List<TransactionsRequestFilter>): Flow<List<TransactionExtended>> {
        return currentWalletId().flatMapLatest { walletId ->
            transactionsDao.getExtendedTransactions(walletId, filters)
        }.mapNotNull { items -> items.toDTO() }
    }

    override fun getTransaction(transactionId: TransactionId): Flow<TransactionExtended?> {
        return currentWalletId().flatMapLatest { walletId ->
            transactionsDao.getExtendedTransaction(walletId, transactionId)
        }.mapNotNull { it?.toDTO() }
            .flowOn(Dispatchers.IO)
    }

    override suspend fun saveTransactions(walletId: WalletId, transactions: List<Transaction>) = withContext(Dispatchers.IO) {
        transactionsDao.insert(transactions.toRecord(walletId))
        transactionsDao.addSwapMetadata(transactions)
    }

    override suspend fun clearPending() {
        transactionsDao.deleteByState(TransactionState.Pending)
    }

    override suspend fun createTransaction(walletId: WalletId, transaction: Transaction): Transaction = withContext(Dispatchers.IO) {
        transactionsDao.insert(listOf(transaction.toRecord(walletId)))
        transactionsDao.addSwapMetadata(listOf(transaction))
        transaction
    }
}
