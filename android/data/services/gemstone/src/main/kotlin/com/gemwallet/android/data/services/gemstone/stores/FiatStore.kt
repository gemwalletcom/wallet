package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.data.service.store.database.FiatTransactionsDao
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.ext.toPrimitives
import com.wallet.core.primitives.FiatTransactionAssetData
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import uniffi.gemstone.FiatTransactionData
import uniffi.gemstone.GemFiatStore

class GemstoneFiatStore(
    private val fiatTransactionsDao: FiatTransactionsDao,
) : GemFiatStore {

    override suspend fun setTransactions(walletId: String, transactions: List<FiatTransactionData>) {
        fiatTransactionsDao.setFiatTransactions(walletId, transactions.map { it.toPrimitives() }.toRecord(walletId))
    }

    fun observeTransactions(walletId: String): Flow<List<FiatTransactionAssetData>> =
        fiatTransactionsDao.getFiatTransactions(walletId).map { it.toDTO() }
}
