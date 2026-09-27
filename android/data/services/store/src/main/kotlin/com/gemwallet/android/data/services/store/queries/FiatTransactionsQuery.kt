package com.gemwallet.android.data.services.store.queries

import com.gemwallet.android.data.services.store.database.FiatTransactionsDao
import com.gemwallet.android.data.services.store.database.entities.toDTO
import com.wallet.core.primitives.FiatTransactionAssetData
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import javax.inject.Inject

class FiatTransactionsQuery @Inject constructor(private val fiatTransactionsDao: FiatTransactionsDao) {

    operator fun invoke(walletId: String): Flow<List<FiatTransactionAssetData>> = fiatTransactionsDao.getFiatTransactions(walletId).map { it.toDTO() }
}
