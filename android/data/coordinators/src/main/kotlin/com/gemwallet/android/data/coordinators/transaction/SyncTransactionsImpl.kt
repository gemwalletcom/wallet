package com.gemwallet.android.data.coordinators.transaction

import android.util.Log
import com.gemwallet.android.application.transactions.cases.SyncTransactions
import com.gemwallet.android.ext.runCatchingCancellable
import javax.inject.Inject
import javax.inject.Singleton
import uniffi.gemstone.GemTransactionsService
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

@Singleton
class SyncTransactionsImpl @Inject constructor(
    private val transactionsService: GemTransactionsService,
) : SyncTransactions {

    override suspend fun syncTransactions(): Boolean = withContext(Dispatchers.IO) {
        runCatchingCancellable { transactionsService.sync(null) }
            .onFailure { Log.e(TAG, "transactions sync failed", it) }
            .isSuccess
    }

    private companion object {
        const val TAG = "SyncTransactions"
    }
}
