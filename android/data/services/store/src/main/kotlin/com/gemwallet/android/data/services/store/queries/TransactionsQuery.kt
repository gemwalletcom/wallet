package com.gemwallet.android.data.services.store.queries

import com.gemwallet.android.application.transactions.values.TransactionsQueryFilter
import com.gemwallet.android.data.services.store.database.TransactionsDao
import com.gemwallet.android.data.services.store.database.entities.toDTO
import com.wallet.core.primitives.TransactionListItem
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map
import javax.inject.Inject

class TransactionsQuery @Inject constructor(private val transactionsDao: TransactionsDao) {

    operator fun invoke(walletId: WalletId, filters: List<TransactionsQueryFilter>, limit: Int): Flow<List<TransactionListItem>> =
        transactionsDao.getTransactionListItems(walletId, filters, limit).distinctUntilChanged().map { items -> items.toDTO() }
}
