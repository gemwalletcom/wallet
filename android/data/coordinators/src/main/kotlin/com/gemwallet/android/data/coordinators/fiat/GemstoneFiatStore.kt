package com.gemwallet.android.data.coordinators.fiat

import com.gemwallet.android.data.service.store.database.FiatTransactionsDao
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.FiatTransactionData
import uniffi.gemstone.GemFiatStore

class GemstoneFiatStore(
    private val fiatTransactionsDao: FiatTransactionsDao,
) : GemFiatStore {

    override suspend fun saveTransactions(walletId: String, transactions: List<String>) {
        fiatTransactionsDao.insert(transactions.map { it.decodeJson<FiatTransactionData>() }.toRecord(walletId))
    }
}
