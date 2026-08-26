package com.gemwallet.android.data.repositories.transactions

import android.util.Log
import com.gemwallet.android.application.transactions.coordinators.TransactionsRequestFilter
import com.gemwallet.android.data.repositories.assets.TransactionPostProcessingService
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.service.store.database.TransactionsDao
import com.gemwallet.android.data.service.store.database.entities.DbTransactionExtended
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.ext.isCompleted
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.TransactionId
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.job
import kotlinx.coroutines.launch
import uniffi.gemstone.GemTransactionStateService
import uniffi.gemstone.transactionStateConfig
import java.util.concurrent.ConcurrentHashMap

private val pollingTransactionStates = listOf(TransactionState.Pending, TransactionState.InTransit)
private const val TAG = "TransactionStateScheduler"

@OptIn(ExperimentalCoroutinesApi::class)
class TransactionStateScheduler(
    private val sessionRepository: SessionRepository,
    private val transactionsDao: TransactionsDao,
    private val stateService: GemTransactionStateService,
    private val postProcessingService: TransactionPostProcessingService,
    private val scope: CoroutineScope = CoroutineScope(SupervisorJob() + Dispatchers.IO),
) {

    private val pollingTransactionJobs = ConcurrentHashMap<TransactionId, Job>()
    private var observeJob: Job? = null

    fun start() {
        if (observeJob != null) return
        observeJob = scope.launch {
            currentWalletId().flatMapLatest { walletId ->
                transactionsDao.getExtendedTransactions(
                    walletId,
                    listOf(TransactionsRequestFilter.States(pollingTransactionStates)),
                )
            }.collect { items ->
                items.forEach { item ->
                    if (!pollingTransactionJobs.containsKey(item.transaction.id)) {
                        val job = pollTransactionStatus(item)
                        pollingTransactionJobs.put(item.transaction.id, job)
                    }
                }
            }
        }
    }

    fun stop() {
        observeJob?.cancel()
        observeJob = null
        pollingTransactionJobs.values.forEach { it.cancel() }
        pollingTransactionJobs.clear()
    }

    private fun currentWalletId(): Flow<WalletId> = sessionRepository.session()
        .filterNotNull()
        .map { it.wallet.id }
        .distinctUntilChanged()

    private fun pollTransactionStatus(transaction: DbTransactionExtended) = scope.launch {
        val jobKeys = mutableSetOf(transaction.transaction.id)
        try {
            var currentTransaction = transaction
            val walletId = transaction.transaction.walletId
            val jobConfig = transactionStateConfig(currentTransaction.transaction.assetId.chain.string)
            var pollingDelay = jobConfig.initialIntervalMs

            while (true) {
                delay(pollingDelay.toLong())
                pollingDelay = jobConfig.nextIntervalMs(pollingDelay)

                val result = try {
                    stateService.update(walletId.id, currentTransaction.transaction.toDTO().toJson())
                } catch (err: Exception) {
                    Log.d(TAG, "transaction status check failed: id=${currentTransaction.transaction.id.identifier}, error=${err.message}")
                    continue
                } ?: return@launch
                val transactionId = result.transactionId.decodeJson<TransactionId>()
                if (transactionId != currentTransaction.transaction.id) {
                    coroutineContext[Job]?.let { runningJob ->
                        pollingTransactionJobs[transactionId] = runningJob
                        jobKeys.add(transactionId)
                    }
                }
                val previousTransaction = currentTransaction
                currentTransaction = transactionsDao.getExtendedTransaction(walletId, transactionId).first() ?: return@launch
                notifyEnteringInTransit(previousTransaction, currentTransaction)

                if (currentTransaction.transaction.state.isCompleted()) {
                    Log.d(
                        TAG,
                        "transaction status complete: id=${currentTransaction.transaction.id.identifier}, state=${currentTransaction.transaction.state}, status=complete",
                    )
                    break
                }
                Log.d(
                    TAG,
                    "transaction status pending: id=${currentTransaction.transaction.id.identifier}, state=${currentTransaction.transaction.state}, next check = ${pollingDelay}ms",
                )
            }
            processChangedTransaction(currentTransaction)
        } finally {
            val runningJob = coroutineContext.job
            jobKeys.forEach { pollingTransactionJobs.remove(it, runningJob) }
        }
    }

    internal suspend fun notifyEnteringInTransit(
        previousTransaction: DbTransactionExtended,
        currentTransaction: DbTransactionExtended,
    ) {
        if (previousTransaction.transaction.state == TransactionState.Pending &&
            currentTransaction.transaction.state == TransactionState.InTransit
        ) {
            processChangedTransaction(currentTransaction)
        }
    }

    private suspend fun processChangedTransaction(transaction: DbTransactionExtended) {
        transaction.toDTO()?.let { postProcessingService.processTransactions(listOf(it)) }
    }
}
