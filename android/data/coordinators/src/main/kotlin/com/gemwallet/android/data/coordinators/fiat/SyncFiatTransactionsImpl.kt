package com.gemwallet.android.data.coordinators.fiat

import android.util.Log
import com.gemwallet.android.application.fiat.cases.SyncFiatTransactions
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.ext.runCatchingCancellable
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.first
import uniffi.gemstone.GemFiatService

class SyncFiatTransactionsImpl(
    private val sessionRepository: SessionRepository,
    private val fiatService: GemFiatService,
) : SyncFiatTransactions {

    override suspend fun invoke(walletId: WalletId?) {
        val resolvedWalletId = walletId ?: sessionRepository.session().first()?.wallet?.id ?: return
        runCatchingCancellable { fiatService.syncTransactions(resolvedWalletId.id) }
            .onFailure { Log.e(TAG, "fiat transactions sync failed for ${resolvedWalletId.id}", it) }
    }

    private companion object {
        const val TAG = "SyncFiatTransactions"
    }
}
