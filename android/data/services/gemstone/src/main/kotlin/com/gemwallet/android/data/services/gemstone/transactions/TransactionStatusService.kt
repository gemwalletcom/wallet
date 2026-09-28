package com.gemwallet.android.data.services.gemstone.transactions

import android.util.Log
import com.gemwallet.android.ext.runCatchingCancellable
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.launch
import uniffi.gemstone.GemTransactionStateService
import uniffi.gemstone.GemTransactionStateServiceInterface
import uniffi.gemstone.GemTransactionStatusService

private const val TAG = "TransactionStatusService"

class TransactionStatusService(
    private val stateService: GemTransactionStateServiceInterface,
    private val ioDispatcher: CoroutineDispatcher = Dispatchers.IO,
    private val scope: CoroutineScope = CoroutineScope(SupervisorJob() + ioDispatcher),
) : GemTransactionStatusService {

    fun start() {
        scope.launch {
            runCatchingCancellable { stateService.trackPending() }
                .onFailure { Log.e(TAG, "pending transactions tracking failed", it) }
        }
    }

    fun stop() {
        stateService.stopTracking()
    }

    override fun track(walletId: String, transactions: List<uniffi.gemstone.Transaction>) {
        scope.launch {
            runCatchingCancellable { stateService.track(walletId, transactions) }
                .onFailure { Log.e(TAG, "tracking failed for $walletId", it) }
        }
    }
}
