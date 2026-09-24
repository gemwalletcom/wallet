package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.application.transactions.cases.TransactionsRequestFilter
import com.gemwallet.android.data.service.store.database.TransactionsDao
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.wallet.core.primitives.TransactionExtended
import com.wallet.core.primitives.TransactionId
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapNotNull

class GemstoneTransactionStore(private val transactionsDao: TransactionsDao) {
    fun observeTransactions(walletId: WalletId, filters: List<TransactionsRequestFilter>): Flow<List<TransactionExtended>> =
        transactionsDao.getExtendedTransactions(walletId, filters).distinctUntilChanged().mapNotNull { items -> items.toDTO() }

    fun observeTransaction(walletId: WalletId, transactionId: TransactionId): Flow<TransactionExtended?> = transactionsDao.getExtendedTransaction(walletId, transactionId).distinctUntilChanged().map { it?.toDTO() }

    fun observeTransactionsCount(walletId: WalletId, filters: List<TransactionsRequestFilter>): Flow<Int?> = transactionsDao.getTransactionsCount(walletId, filters).distinctUntilChanged()
}
