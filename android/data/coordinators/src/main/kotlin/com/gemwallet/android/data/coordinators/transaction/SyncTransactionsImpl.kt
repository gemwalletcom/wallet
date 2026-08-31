package com.gemwallet.android.data.coordinators.transaction

import android.util.Log
import com.gemwallet.android.application.transactions.cases.SyncAssetTransactions
import com.gemwallet.android.application.transactions.cases.SyncTransactions
import com.gemwallet.android.application.session.cases.GetCurrentWallet
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Wallet
import javax.inject.Inject
import javax.inject.Singleton
import uniffi.gemstone.GemTransactionsService

@Singleton
class SyncTransactionsImpl @Inject constructor(
    private val transactionsService: GemTransactionsService,
    private val getCurrentWallet: GetCurrentWallet,
) : SyncTransactions, SyncAssetTransactions {

    override suspend fun syncTransactions(wallet: Wallet): Boolean =
        runCatchingCancellable { transactionsService.sync(wallet.id.id, null) }
            .onFailure { Log.e(TAG, "transactions sync failed for ${wallet.id.id}", it) }
            .isSuccess

    override suspend fun syncAssetTransactions(assetId: AssetId) {
        val wallet = getCurrentWallet.getCurrentWallet() ?: return
        runCatchingCancellable { transactionsService.sync(wallet.id.id, assetId.toIdentifier()) }
            .onFailure { Log.e(TAG, "asset transactions sync failed for ${assetId.toIdentifier()}", it) }
    }

    private companion object {
        const val TAG = "SyncTransactions"
    }
}
