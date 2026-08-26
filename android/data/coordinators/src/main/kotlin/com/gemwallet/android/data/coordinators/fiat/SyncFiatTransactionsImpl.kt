package com.gemwallet.android.data.coordinators.fiat

import com.gemwallet.android.application.fiat.coordinators.SyncFiatTransactions
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.first
import uniffi.gemstone.GemFiatService

class SyncFiatTransactionsImpl(
    private val sessionRepository: SessionRepository,
    private val fiatService: GemFiatService,
) : SyncFiatTransactions {

    override suspend fun invoke(walletId: WalletId?) {
        val resolvedWalletId = walletId ?: sessionRepository.session().first()?.wallet?.id ?: return
        runCatching { fiatService.syncTransactions(resolvedWalletId.id) }
    }
}
