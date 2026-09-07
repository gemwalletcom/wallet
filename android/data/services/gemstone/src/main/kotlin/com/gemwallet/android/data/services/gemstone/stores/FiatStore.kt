package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.data.service.store.database.FiatTransactionsDao
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.wallet.core.primitives.FiatTransactionAssetData
import com.wallet.core.primitives.FiatTransactionData
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import uniffi.gemstone.GemFiatStore

class GemstoneFiatStore(
    private val fiatTransactionsDao: FiatTransactionsDao,
) : GemFiatStore {

    override suspend fun setTransactions(walletId: String, transactions: List<String>) {
        fiatTransactionsDao.setFiatTransactions(walletId, transactions.map { it.decodeJson<FiatTransactionData>() }.toRecord(walletId))
    }

    fun observeTransactions(walletId: String): Flow<List<FiatTransactionAssetData>> =
        fiatTransactionsDao.getFiatTransactions(walletId).map { it.toDTO() }
}
