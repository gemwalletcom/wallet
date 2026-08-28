package com.gemwallet.android.data.repositories.assets

import android.util.Log
import com.gemwallet.android.ext.runCatchingCancellable
import uniffi.gemstone.GemBalanceService

class UpdateBalances(
    private val balanceService: GemBalanceService,
) {

    suspend fun updateBalances(walletId: String, assetIds: List<String>) {
        runCatchingCancellable { balanceService.update(walletId, assetIds) }
            .onFailure { Log.e(TAG, "balances update failed for $walletId", it) }
    }

    private companion object {
        const val TAG = "UpdateBalances"
    }
}
