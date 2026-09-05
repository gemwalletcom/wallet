package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.data.service.store.database.StoreTransactionRunner
import com.gemwallet.android.data.service.store.database.TransactionsDao
import com.gemwallet.android.data.services.gemstone.transactions.addSwapMetadata
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.TransactionId
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.WalletId
import com.gemwallet.android.data.service.store.database.entities.DbTransaction
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.wallet.core.primitives.Transaction
import uniffi.gemstone.GemPendingTransaction
import uniffi.gemstone.GemTransactionStateStore
import uniffi.gemstone.GemTransactionStateUpdate

class GemstoneTransactionStateStore(
    private val transactionsDao: TransactionsDao,
    private val walletStore: GemstoneWalletStore,
    private val transactionRunner: StoreTransactionRunner,
) : GemTransactionStateStore {
    override suspend fun getPendingTransactions(): List<GemPendingTransaction> {
        val records = transactionsDao.getTransactionsByStates(listOf(TransactionState.Pending, TransactionState.InTransit))
        if (records.isEmpty()) {
            return emptyList()
        }
        val wallets = walletStore.getAllNow().associateBy { it.id }
        return records.mapNotNull { record ->
            val wallet = wallets[record.walletId] ?: return@mapNotNull null
            GemPendingTransaction(wallet = wallet.toGem(), transaction = record.toDTO().toJson())
        }
    }

    override suspend fun getTransaction(walletId: String, transactionId: String): GemPendingTransaction? =
        transactionsDao.getTransaction(TransactionId(transactionId), WalletId(walletId))?.let { pendingTransaction(it) }

    override suspend fun addTransactions(walletId: String, transactions: List<String>) {
        val records = transactions.map { it.decodeJson<Transaction>() }
        transactionRunner.run {
            transactionsDao.insert(records.map { it.toRecord(WalletId(walletId)) })
            transactionsDao.addSwapMetadata(records)
        }
    }

    private suspend fun pendingTransaction(record: DbTransaction): GemPendingTransaction? {
        val wallet = walletStore.getWalletNow(record.walletId) ?: return null
        return GemPendingTransaction(wallet = wallet.toGem(), transaction = record.toDTO().toJson())
    }


    override suspend fun getState(walletId: String, transactionId: String): uniffi.gemstone.TransactionState? =
        transactionsDao.getTransactionState(TransactionId(transactionId), WalletId(walletId))?.toGem()

    override suspend fun renameTransaction(walletId: String, transactionId: String, newTransactionId: String) {
        val oldId = TransactionId(transactionId)
        val newId = TransactionId(newTransactionId)
        val wallet = WalletId(walletId)
        transactionRunner.run {
            transactionsDao.updateTransactionId(oldId, newId, wallet, newId.hash)
            transactionsDao.getTransaction(newId, wallet)?.let { transactionsDao.addSwapMetadata(listOf(it.toDTO())) }
            transactionsDao.deleteUnreferencedSwapMetadata(oldId.identifier)
        }
    }

    override suspend fun deleteTransaction(walletId: String, transactionId: String) {
        val id = TransactionId(transactionId)
        transactionRunner.run {
            transactionsDao.delete(id, WalletId(walletId))
            transactionsDao.deleteUnreferencedSwapMetadata(id.identifier)
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
            if (updatedRows > 0 && update.metadata != null) {
                transactionsDao.getTransaction(id, wallet)?.let { transactionsDao.addSwapMetadata(listOf(it.toDTO())) }
            }
            updatedRows > 0
        }
    }
}
