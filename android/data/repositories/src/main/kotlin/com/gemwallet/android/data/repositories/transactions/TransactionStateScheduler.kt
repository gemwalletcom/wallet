package com.gemwallet.android.data.repositories.transactions

import android.util.Log
import com.gemwallet.android.cases.transactions.CreateTransaction
import com.gemwallet.android.ext.isCompleted
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.Transaction
import com.wallet.core.primitives.TransactionId
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.delay
import kotlinx.coroutines.job
import kotlinx.coroutines.launch
import uniffi.gemstone.GemPendingTransaction
import uniffi.gemstone.GemTransactionStateService
import uniffi.gemstone.transactionStateConfig
import java.util.concurrent.ConcurrentHashMap

private const val TAG = "TransactionStateScheduler"

class TransactionStateScheduler(
    private val stateService: GemTransactionStateService,
    private val scope: CoroutineScope = CoroutineScope(SupervisorJob() + Dispatchers.IO),
) : CreateTransaction {

    private val pollingTransactionJobs = ConcurrentHashMap<TransactionId, Job>()

    fun start() {
        scope.launch {
            val pending = try {
                stateService.pendingTransactions()
            } catch (err: Exception) {
                Log.d(TAG, "pending transactions load failed: ${err.message}")
                return@launch
            }
            pending.forEach { schedule(it.walletId(), it.transaction.decodeJson()) }
        }
    }

    fun stop() {
        pollingTransactionJobs.values.forEach { it.cancel() }
        pollingTransactionJobs.clear()
    }

    override suspend fun createTransaction(walletId: WalletId, transaction: Transaction, currency: Currency): Transaction {
        stateService.addTransactions(walletId.id, listOf(transaction.toJson()), currency.toJson()).forEach { failure ->
            Log.e(TAG, "${failure.step} failed after adding ${transaction.id.hash}: ${failure.message}")
        }
        schedule(walletId, transaction)
        return transaction
    }

    private fun schedule(walletId: WalletId, transaction: Transaction) {
        pollingTransactionJobs.computeIfAbsent(transaction.id) { pollTransactionStatus(walletId, transaction) }
    }

    private fun pollTransactionStatus(walletId: WalletId, transaction: Transaction) = scope.launch {
        val jobKeys = mutableSetOf(transaction.id)
        try {
            var currentTransaction = transaction
            val jobConfig = transactionStateConfig(currentTransaction.assetId.chain.string)
            var pollingDelay = jobConfig.initialIntervalMs
            while (true) {
                delay(pollingDelay.toLong())
                pollingDelay = jobConfig.nextIntervalMs(pollingDelay)
                val result = try {
                    stateService.update(walletId.id, currentTransaction.toJson())
                } catch (err: Exception) {
                    Log.d(TAG, "transaction status check failed: id=${currentTransaction.id.identifier}, error=${err.message}")
                    continue
                } ?: return@launch
                result.failures.forEach { failure ->
                    Log.d(TAG, "transaction post-processing ${failure.step} failed: id=${currentTransaction.id.identifier}, error=${failure.message}")
                }
                val transactionId = result.transactionId.decodeJson<TransactionId>()
                if (transactionId != currentTransaction.id) {
                    pollingTransactionJobs[transactionId] = coroutineContext.job
                    jobKeys.add(transactionId)
                }
                currentTransaction = stateService.getTransaction(walletId.id, transactionId.toJson())?.transaction?.decodeJson() ?: return@launch
                if (currentTransaction.state.isCompleted()) {
                    Log.d(TAG, "transaction status complete: id=${currentTransaction.id.identifier}, state=${currentTransaction.state}")
                    break
                }
                Log.d(TAG, "transaction status pending: id=${currentTransaction.id.identifier}, state=${currentTransaction.state}, next check = ${pollingDelay}ms")
            }
        } finally {
            val runningJob = coroutineContext.job
            jobKeys.forEach { pollingTransactionJobs.remove(it, runningJob) }
        }
    }
}

private fun GemPendingTransaction.walletId(): WalletId = wallet.decodeJson<Wallet>().id
