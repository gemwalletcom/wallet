package com.gemwallet.android.data.services.store.queries

import com.gemwallet.android.data.services.store.database.TransactionsDao
import com.wallet.core.primitives.TransactionsFilter
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.distinctUntilChanged
import javax.inject.Inject

class TransactionsCountQuery @Inject constructor(private val transactionsDao: TransactionsDao) {

    operator fun invoke(walletId: WalletId, filter: TransactionsFilter?): Flow<Int?> = transactionsDao.getTransactionsCount(walletId, filter).distinctUntilChanged()
}
