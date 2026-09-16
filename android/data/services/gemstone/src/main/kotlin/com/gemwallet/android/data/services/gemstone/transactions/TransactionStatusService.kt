package com.gemwallet.android.data.services.gemstone.transactions

import com.gemwallet.android.ext.toGem
import android.util.Log
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.application.transactions.cases.CreateTransaction
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Transaction
import com.wallet.core.primitives.Wallet
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemTransactionStateService
import uniffi.gemstone.GemTransactionStateServiceInterface
import uniffi.gemstone.GemTransactionStatusService

private const val TAG = "TransactionStatusService"

class TransactionStatusService(
    private val stateService: GemTransactionStateServiceInterface,
    private val ioDispatcher: CoroutineDispatcher = Dispatchers.IO,
    private val scope: CoroutineScope = CoroutineScope(SupervisorJob() + ioDispatcher),
) : CreateTransaction, GemTransactionStatusService {

    fun start() {
        scope.launch {
            runCatchingCancellable { stateService.trackPending() }
                .onFailure { Log.e(TAG, "pending transactions tracking failed", it) }
        }
    }

    fun stop() {
        stateService.stopTracking()
    }

    override suspend fun createNotificationTransaction(wallet: Wallet, assetId: AssetId, transaction: Transaction): Asset? = withContext(ioDispatcher) {
        val asset = stateService.addNotificationTransaction(wallet.toGem(), assetId.toIdentifier(), transaction.toGem())
            ?.toPrimitives() ?: return@withContext null
        track(wallet.id.id, listOf(transaction.toGem()))
        asset
    }

    override fun track(walletId: String, transactions: List<uniffi.gemstone.Transaction>) {
        scope.launch {
            runCatchingCancellable { stateService.track(walletId, transactions) }
                .onFailure { Log.e(TAG, "tracking failed for $walletId", it) }
        }
    }
}
