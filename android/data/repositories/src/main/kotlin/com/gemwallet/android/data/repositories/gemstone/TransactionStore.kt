package com.gemwallet.android.data.repositories.gemstone

import com.gemwallet.android.data.service.store.database.StoreTransactionRunner
import com.gemwallet.android.data.service.store.database.TransactionsDao
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.data.repositories.transactions.addSwapMetadata
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.Transaction
import com.wallet.core.primitives.WalletId
import uniffi.gemstone.GemTransactionStore

class GemstoneTransactionStore(
    private val transactionsDao: TransactionsDao,
    private val transactionRunner: StoreTransactionRunner,
) : GemTransactionStore {
    override suspend fun saveTransactions(walletId: String, transactions: List<String>) {
        val records = transactions.map { it.decodeJson<Transaction>() }
        transactionRunner.run {
            transactionsDao.insert(records.toRecord(WalletId(walletId)))
            transactionsDao.addSwapMetadata(records)
        }
    }
}
