package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.data.services.store.database.StoreTransactionRunner
import com.gemwallet.android.data.services.store.database.TransactionsDao
import com.gemwallet.android.data.services.store.database.entities.DbTransaction
import com.gemwallet.android.data.services.store.database.entities.toDTO
import com.gemwallet.android.data.services.store.database.entities.toRecord
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.wallet.core.primitives.TransactionId
import com.wallet.core.primitives.WalletId
import uniffi.gemstone.GemPendingTransaction
import uniffi.gemstone.GemTransactionStateStore
import uniffi.gemstone.GemTransactionStateUpdate
import uniffi.gemstone.transactionAssetIds

class GemstoneTransactionStateStore(private val transactionsDao: TransactionsDao, private val transactionRunner: StoreTransactionRunner) : GemTransactionStateStore {
    override suspend fun getPendingTransactions(states: List<uniffi.gemstone.TransactionState>): List<GemPendingTransaction> = transactionsDao.getTransactionsByStates(states.map { it.toPrimitives() }).map(::pendingTransaction)

    override suspend fun getTransaction(walletId: String, transactionId: String): GemPendingTransaction? = transactionsDao.getTransaction(TransactionId(transactionId), WalletId(walletId))?.let { pendingTransaction(it) }

    override suspend fun addTransactions(walletId: String, transactions: List<uniffi.gemstone.Transaction>) {
        transactionRunner.run {
            transactionsDao.insert(transactions.map { it.toPrimitives().toRecord(WalletId(walletId)) })
            transactionsDao.replaceTransactionAssets(transactions.associate { it.id to transactionAssetIds(it) })
        }
    }

    private fun pendingTransaction(record: DbTransaction) = GemPendingTransaction(walletId = record.walletId.id, transaction = record.toDTO().toGem())

    override suspend fun getState(walletId: String, transactionId: String): uniffi.gemstone.TransactionState? = transactionsDao.getTransactionState(TransactionId(transactionId), WalletId(walletId))?.toGem()

    override suspend fun updateTransactionHash(walletId: String, transactionId: String, hash: String) {
        val oldId = TransactionId(transactionId)
        val wallet = WalletId(walletId)
        transactionRunner.run {
            transactionsDao.updateTransactionHash(oldId, wallet, hash)
            transactionsDao.deleteUnreferencedTransactionAssets(oldId.identifier)
        }
    }

    override suspend fun deleteTransaction(walletId: String, transactionId: String) {
        val id = TransactionId(transactionId)
        transactionRunner.run {
            transactionsDao.delete(id, WalletId(walletId))
            transactionsDao.deleteUnreferencedTransactionAssets(id.identifier)
        }
    }

    override suspend fun updateTransaction(walletId: String, transactionId: String, update: GemTransactionStateUpdate): Boolean {
        val id = TransactionId(transactionId)
        val wallet = WalletId(walletId)
        return transactionRunner.run {
            val updatedRows = transactionsDao.updateTransactionState(
                id = id,
                walletId = wallet,
                state = update.state.toPrimitives(),
                fee = update.fee?.toString(),
                blockNumber = update.blockNumber,
                metadata = update.metadata,
                confirmationEtaSeconds = update.confirmationEtaSeconds?.toLong(),
            )
            if (updatedRows > 0) update.assetIds?.let { transactionsDao.replaceTransactionAssets(mapOf(id.identifier to it)) }
            updatedRows > 0
        }
    }
}
