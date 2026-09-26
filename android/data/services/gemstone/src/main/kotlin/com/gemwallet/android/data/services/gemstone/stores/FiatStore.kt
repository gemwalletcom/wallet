package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.data.services.store.database.FiatTransactionsDao
import com.gemwallet.android.data.services.store.database.entities.toRecord
import com.gemwallet.android.ext.toPrimitives
import uniffi.gemstone.FiatTransactionData
import uniffi.gemstone.GemFiatStore

class GemstoneFiatStore(private val fiatTransactionsDao: FiatTransactionsDao) : GemFiatStore {

    override suspend fun setTransactions(walletId: String, transactions: List<FiatTransactionData>) {
        fiatTransactionsDao.setFiatTransactions(walletId, transactions.map { it.toPrimitives() }.toRecord(walletId))
    }
}
