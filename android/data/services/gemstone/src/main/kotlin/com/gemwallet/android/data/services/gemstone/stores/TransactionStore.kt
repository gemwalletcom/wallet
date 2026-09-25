package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.application.transactions.cases.TransactionsRequestFilter
import com.gemwallet.android.data.services.store.database.TransactionsDao
import com.gemwallet.android.data.services.store.database.entities.toDTO
import com.gemwallet.android.ext.GemConstants
import com.wallet.core.primitives.TransactionExtended
import com.wallet.core.primitives.TransactionId
import com.wallet.core.primitives.TransactionListItem
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map

class GemstoneTransactionStore(private val transactionsDao: TransactionsDao) {
    fun observeTransactions(walletId: WalletId, filters: List<TransactionsRequestFilter>): Flow<List<TransactionListItem>> =
        transactionsDao.getTransactionListItems(walletId, filters, GemConstants.transactionsListLimit).distinctUntilChanged().map { items -> items.toDTO() }

    fun observeTransaction(walletId: WalletId, transactionId: TransactionId): Flow<TransactionExtended?> = transactionsDao.getExtendedTransaction(walletId, transactionId).distinctUntilChanged().map { it?.toDTO() }

    fun observeTransactionsCount(walletId: WalletId, filters: List<TransactionsRequestFilter>): Flow<Int?> = transactionsDao.getTransactionsCount(walletId, filters).distinctUntilChanged()
}
