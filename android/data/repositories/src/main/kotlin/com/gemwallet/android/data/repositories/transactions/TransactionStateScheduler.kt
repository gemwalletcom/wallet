package com.gemwallet.android.data.repositories.transactions

import android.util.Log
import com.gemwallet.android.cases.transactions.CreateTransaction
import com.gemwallet.android.ext.isCompleted
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
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
        trackPendingTransactions()
    }

    override fun trackPendingTransactions() {
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

    override suspend fun trackTransactions(walletId: WalletId, transactions: List<Transaction>) {
        track(walletId, transactions)
    }

    override suspend fun createNotificationTransaction(wallet: Wallet, assetId: AssetId, transaction: Transaction): Asset? {
        val asset = stateService.addNotificationTransaction(wallet.toJson(), assetId.toIdentifier(), transaction.toJson())
            ?.decodeJson<Asset>() ?: return null
        track(wallet.id, listOf(transaction))
        return asset
    }

    private fun track(walletId: WalletId, transactions: List<Transaction>) {
        transactions.forEach { schedule(walletId, it) }
        scope.launch {
            runCatchingCancellable { stateService.enableTransactionAssets(walletId.id, transactions.map { it.toJson() }) }
                .onFailure { Log.e(TAG, "asset enabling failed after adding ${transactions.map { it.id.hash }}", it) }
        }
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
