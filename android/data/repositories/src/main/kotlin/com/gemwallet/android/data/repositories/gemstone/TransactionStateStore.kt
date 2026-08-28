package com.gemwallet.android.data.repositories.gemstone

import com.gemwallet.android.data.service.store.database.StoreTransactionRunner
import com.gemwallet.android.data.service.store.database.TransactionsDao
import com.gemwallet.android.data.repositories.transactions.addSwapMetadata
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.TransactionId
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.WalletId
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.data.service.store.database.entities.DbTransaction
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.wallet.core.primitives.Transaction
import dagger.Lazy
import uniffi.gemstone.GemPendingTransaction
import uniffi.gemstone.GemTransactionStateStore
import uniffi.gemstone.GemTransactionStateUpdate

class GemstoneTransactionStateStore(
    private val transactionsDao: TransactionsDao,
    private val walletsRepository: Lazy<WalletsRepository>,
    private val transactionRunner: StoreTransactionRunner,
) : GemTransactionStateStore {
    override suspend fun getPendingTransactions(): List<GemPendingTransaction> {
        val records = transactionsDao.getTransactionsByStates(listOf(TransactionState.Pending, TransactionState.InTransit))
        if (records.isEmpty()) {
            return emptyList()
        }
        val wallets = walletsRepository.get().getAllNow().associateBy { it.id }
        return records.mapNotNull { record ->
            val wallet = wallets[record.walletId] ?: return@mapNotNull null
            GemPendingTransaction(wallet = wallet.toJson(), transaction = record.toDTO().toJson())
        }
    }

    override suspend fun getTransaction(walletId: String, transactionId: String): GemPendingTransaction? =
        transactionsDao.getTransaction(transactionId.decodeJson(), WalletId(walletId))?.let { pendingTransaction(it) }

    override suspend fun addTransactions(walletId: String, transactions: List<String>) {
        val records = transactions.map { it.decodeJson<Transaction>() }
        transactionRunner.run {
            transactionsDao.insert(records.map { it.toRecord(WalletId(walletId)) })
            transactionsDao.addSwapMetadata(records)
        }
    }

    private fun pendingTransaction(record: DbTransaction): GemPendingTransaction? {
        val wallet = walletsRepository.get().getWalletNow(record.walletId) ?: return null
        return GemPendingTransaction(wallet = wallet.toJson(), transaction = record.toDTO().toJson())
    }


    override suspend fun getState(walletId: String, transactionId: String): String? =
        transactionsDao.getTransactionState(transactionId.decodeJson(), WalletId(walletId))?.toJson()

    override suspend fun renameTransaction(walletId: String, transactionId: String, newTransactionId: String) {
        val oldId = transactionId.decodeJson<TransactionId>()
        val newId = newTransactionId.decodeJson<TransactionId>()
        val wallet = WalletId(walletId)
        transactionRunner.run {
            transactionsDao.updateTransactionId(oldId, newId, wallet, newId.hash)
            transactionsDao.getTransaction(newId, wallet)?.let { transactionsDao.addSwapMetadata(listOf(it.toDTO())) }
            transactionsDao.deleteUnreferencedSwapMetadata(oldId.identifier)
        }
    }

    override suspend fun deleteTransaction(walletId: String, transactionId: String) {
        val id = transactionId.decodeJson<TransactionId>()
        transactionRunner.run {
            transactionsDao.delete(id, WalletId(walletId))
            transactionsDao.deleteUnreferencedSwapMetadata(id.identifier)
        }
    }

    override suspend fun updateTransaction(walletId: String, transactionId: String, update: GemTransactionStateUpdate): Boolean {
        val id = transactionId.decodeJson<TransactionId>()
        val wallet = WalletId(walletId)
        return transactionRunner.run {
            val updatedRows = transactionsDao.updateTransactionState(
                id = id,
                walletId = wallet,
                state = update.state.decodeJson<TransactionState>(),
                fee = update.fee,
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
