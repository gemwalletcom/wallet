package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.data.service.store.database.StoreTransactionRunner
import com.gemwallet.android.application.transactions.cases.TransactionsRequestFilter
import com.gemwallet.android.data.service.store.database.TransactionsDao
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.wallet.core.primitives.TransactionExtended
import com.wallet.core.primitives.TransactionId
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.mapNotNull
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.data.services.gemstone.transactions.addSwapMetadata
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.Transaction
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

    fun observeTransactions(walletId: WalletId, filters: List<TransactionsRequestFilter>): Flow<List<TransactionExtended>> =
        transactionsDao.getExtendedTransactions(walletId, filters).mapNotNull { items -> items.toDTO() }

    fun observeTransaction(walletId: WalletId, transactionId: TransactionId): Flow<TransactionExtended?> =
        transactionsDao.getExtendedTransaction(walletId, transactionId).mapNotNull { it?.toDTO() }

    fun observeTransactionsCount(walletId: WalletId, filters: List<TransactionsRequestFilter>): Flow<Int?> =
        transactionsDao.getTransactionsCount(walletId, filters)

}
