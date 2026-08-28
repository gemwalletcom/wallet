package com.gemwallet.android.data.repositories.transactions

import android.util.Log
import com.gemwallet.android.cases.transactions.CreateTransaction
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Transaction
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.launch
import uniffi.gemstone.GemTransactionStateService

private const val TAG = "TransactionStateTracker"

class TransactionStateTracker(
    private val stateService: GemTransactionStateService,
    private val scope: CoroutineScope = CoroutineScope(SupervisorJob() + Dispatchers.IO),
) : CreateTransaction {

    fun start() {
        trackPendingTransactions()
    }

    fun stop() {
        stateService.stopTracking()
    }

    override fun trackPendingTransactions() {
        scope.launch {
            runCatchingCancellable { stateService.trackPending() }
                .onFailure { Log.e(TAG, "pending transactions tracking failed", it) }
        }
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
        scope.launch {
            runCatchingCancellable { stateService.track(walletId.id, transactions.map { it.toJson() }) }
                .onFailure { Log.e(TAG, "tracking failed for ${transactions.map { it.id.hash }}", it) }
        }
    }
}
