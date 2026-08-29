package com.gemwallet.android.data.coordinators.transaction

import com.gemwallet.android.application.transactions.cases.GetPendingTransactionsCount
import com.gemwallet.android.application.transactions.cases.TransactionsRequestFilter
import com.gemwallet.android.cases.transactions.ClearPendingTransactions
import com.gemwallet.android.application.session.cases.GetCurrentWalletId
import com.gemwallet.android.data.repositories.gemstone.GemstoneTransactionStore
import com.wallet.core.primitives.TransactionState
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flatMapLatest

private val pendingTransactionStates = listOf(TransactionState.Pending, TransactionState.InTransit)

@OptIn(ExperimentalCoroutinesApi::class)
class GetPendingTransactionsCountImpl(
    private val getCurrentWalletId: GetCurrentWalletId,
    private val transactionStore: GemstoneTransactionStore,
) : GetPendingTransactionsCount {

    override fun getPendingTransactionsCount(): Flow<Int?> = getCurrentWalletId()
        .flatMapLatest { walletId ->
            transactionStore.observeTransactionsCount(
                walletId,
                TransactionsRequestFilter.activityDefaults() + TransactionsRequestFilter.States(pendingTransactionStates),
            )
        }
}

class ClearPendingTransactionsImpl(
    private val transactionStore: GemstoneTransactionStore,
) : ClearPendingTransactions {

    override suspend fun clearPending() {
        transactionStore.deletePending(TransactionState.Pending)
    }
}
