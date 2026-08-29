package com.gemwallet.android.data.coordinators.transaction

import com.gemwallet.android.application.transactions.cases.TransactionsRequestFilter
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.gemstone.GemstoneTransactionStore
import com.gemwallet.android.model.TransactionExtended
import com.wallet.core.primitives.TransactionId
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map

@OptIn(ExperimentalCoroutinesApi::class)
internal fun SessionRepository.currentWalletId(): Flow<WalletId> = session()
    .filterNotNull()
    .map { it.wallet.id }
    .distinctUntilChanged()

@OptIn(ExperimentalCoroutinesApi::class)
internal fun GemstoneTransactionStore.walletTransactions(
    sessionRepository: SessionRepository,
    filters: List<TransactionsRequestFilter>,
): Flow<List<TransactionExtended>> = sessionRepository.currentWalletId()
    .flatMapLatest { walletId -> observeTransactions(walletId, filters) }

@OptIn(ExperimentalCoroutinesApi::class)
internal fun GemstoneTransactionStore.walletTransaction(
    sessionRepository: SessionRepository,
    transactionId: TransactionId,
): Flow<TransactionExtended?> = sessionRepository.currentWalletId()
    .flatMapLatest { walletId -> observeTransaction(walletId, transactionId) }
    .flowOn(Dispatchers.IO)
