package com.gemwallet.android.data.coordinators.fiat

import android.util.Log
import com.gemwallet.android.application.fiat.cases.SyncFiatTransactions
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.ext.runCatchingCancellable
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.first
import uniffi.gemstone.GemFiatService

class SyncFiatTransactionsImpl(
    private val getSession: GetSession,
    private val fiatService: GemFiatService,
) : SyncFiatTransactions {

    override suspend fun invoke(walletId: WalletId?) {
        val resolvedWalletId = walletId ?: getSession().first()?.wallet?.id ?: return
        runCatchingCancellable { fiatService.syncTransactions(resolvedWalletId.id) }
            .onFailure { Log.e(TAG, "fiat transactions sync failed for ${resolvedWalletId.id}", it) }
    }

    private companion object {
        const val TAG = "SyncFiatTransactions"
    }
}
