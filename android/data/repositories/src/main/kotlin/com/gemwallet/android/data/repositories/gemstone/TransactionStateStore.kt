package com.gemwallet.android.data.repositories.gemstone

import com.gemwallet.android.data.service.store.database.TransactionsDao
import com.gemwallet.android.data.repositories.transactions.addSwapMetadata
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.TransactionId
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.WalletId
import uniffi.gemstone.GemTransactionStateStore
import uniffi.gemstone.GemTransactionStateUpdate

class GemstoneTransactionStateStore(
    private val transactionsDao: TransactionsDao,
) : GemTransactionStateStore {

    override suspend fun getState(walletId: String, transactionId: String): String? =
        transactionsDao.getTransactionState(transactionId.decodeJson(), WalletId(walletId))?.toJson()

    override suspend fun renameTransaction(walletId: String, transactionId: String, newTransactionId: String) {
        val oldId = transactionId.decodeJson<TransactionId>()
        val newId = newTransactionId.decodeJson<TransactionId>()
        transactionsDao.updateSwapMetadataTransactionId(oldId.identifier, newId.identifier)
        transactionsDao.updateTransactionId(oldId, newId, WalletId(walletId), newId.hash)
    }

    override suspend fun deleteTransaction(walletId: String, transactionId: String) {
        val id = transactionId.decodeJson<TransactionId>()
        transactionsDao.deleteSwapMetadata(id.identifier)
        transactionsDao.delete(id, WalletId(walletId))
    }

    override suspend fun updateTransaction(walletId: String, transactionId: String, update: GemTransactionStateUpdate): Boolean {
        val id = transactionId.decodeJson<TransactionId>()
        val wallet = WalletId(walletId)
        val updatedRows = transactionsDao.updateTransactionState(
            id = id,
            walletId = wallet,
            state = update.state.decodeJson<TransactionState>(),
            fee = update.fee,
            blockNumber = update.blockNumber,
            metadata = update.metadata,
            confirmationEtaSeconds = update.confirmationEtaSeconds?.toLong(),
        )
        if (updatedRows == 0) return false
        if (update.metadata != null) {
            transactionsDao.getTransaction(id, wallet)?.let { transactionsDao.addSwapMetadata(listOf(it.toDTO())) }
        }
        return true
    }
}
