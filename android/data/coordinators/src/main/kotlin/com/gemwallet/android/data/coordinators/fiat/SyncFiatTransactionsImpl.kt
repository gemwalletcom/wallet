package com.gemwallet.android.data.coordinators.fiat

import com.gemwallet.android.application.assets.coordinators.PrefetchAssets
import com.gemwallet.android.application.fiat.coordinators.GetFiatTransactions
import com.gemwallet.android.application.fiat.coordinators.SyncFiatTransactions
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.service.store.database.FiatTransactionsDao
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.wallet.core.primitives.FiatTransactionData
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.flow.first

class SyncFiatTransactionsImpl(
    private val sessionRepository: SessionRepository,
    private val getFiatTransactions: GetFiatTransactions,
    private val prefetchAssets: PrefetchAssets,
    private val fiatTransactionsDao: FiatTransactionsDao,
) : SyncFiatTransactions {

    override suspend fun invoke(walletId: WalletId?) {
        val resolvedWalletId = walletId ?: WalletId(sessionRepository.session().first()?.wallet?.id ?: return)
        try {
            val transactions = getFiatTransactions(resolvedWalletId)
            prefetchAssets(transactions)
            fiatTransactionsDao.insert(transactions.toRecord(resolvedWalletId.id))
        } catch (_: Exception) {
            currentCoroutineContext().ensureActive()
        }
    }

    private suspend fun prefetchAssets(transactions: List<FiatTransactionData>) {
        val assetIds = transactions
            .map { it.transaction.assetId }
            .distinct()

        prefetchAssets.prefetchAssets(assetIds)
    }
}
