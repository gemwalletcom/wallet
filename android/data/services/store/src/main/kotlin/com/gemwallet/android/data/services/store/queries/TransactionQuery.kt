package com.gemwallet.android.data.services.store.queries

import com.gemwallet.android.data.services.store.database.TransactionsDao
import com.gemwallet.android.data.services.store.database.entities.toDTO
import com.wallet.core.primitives.TransactionExtended
import com.wallet.core.primitives.TransactionId
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map
import javax.inject.Inject

class TransactionQuery @Inject constructor(private val transactionsDao: TransactionsDao) {

    operator fun invoke(walletId: WalletId, transactionId: TransactionId): Flow<TransactionExtended?> = transactionsDao.getExtendedTransaction(walletId, transactionId).distinctUntilChanged().map { it?.toDTO() }
}
