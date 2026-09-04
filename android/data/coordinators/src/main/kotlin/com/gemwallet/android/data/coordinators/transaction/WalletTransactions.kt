package com.gemwallet.android.data.coordinators.transaction

import com.gemwallet.android.application.transactions.cases.TransactionsRequestFilter
import com.gemwallet.android.application.session.cases.GetCurrentWalletId
import com.gemwallet.android.data.services.gemstone.stores.GemstoneTransactionStore
import com.wallet.core.primitives.TransactionExtended
import com.wallet.core.primitives.TransactionId
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOn

@OptIn(ExperimentalCoroutinesApi::class)
internal fun GemstoneTransactionStore.walletTransactions(
    getCurrentWalletId: GetCurrentWalletId,
    filters: List<TransactionsRequestFilter>,
): Flow<List<TransactionExtended>> = getCurrentWalletId()
    .flatMapLatest { walletId -> observeTransactions(walletId, filters) }

@OptIn(ExperimentalCoroutinesApi::class)
internal fun GemstoneTransactionStore.walletTransaction(
    getCurrentWalletId: GetCurrentWalletId,
    transactionId: TransactionId,
): Flow<TransactionExtended?> = getCurrentWalletId()
    .flatMapLatest { walletId -> observeTransaction(walletId, transactionId) }
    .flowOn(Dispatchers.IO)
