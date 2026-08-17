package com.gemwallet.android.data.repositories.transactions

import android.text.format.DateUtils
import com.gemwallet.android.blockchain.model.ServiceUnavailable
import com.gemwallet.android.blockchain.services.TransactionStatusService
import com.gemwallet.android.data.service.store.database.TransactionsDao
import com.gemwallet.android.data.service.store.database.entities.DbTransaction
import com.gemwallet.android.data.service.store.database.entities.DbTransactionExtended
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.ext.getTransactionSwapMetadata
import com.gemwallet.android.ext.isCompleted
import com.gemwallet.android.ext.toSwapProvider
import com.wallet.core.primitives.TransactionId
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.TransactionStateRequest
import com.wallet.core.primitives.TransactionSwapStateRequest
import com.wallet.core.primitives.TransactionType
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import uniffi.gemstone.Config

class TransactionStateService(
    private val transactionsDao: TransactionsDao,
    private val transactionStatusService: TransactionStatusService,
) {

    internal suspend fun checkTransaction(transaction: DbTransactionExtended): DbTransactionExtended? {
        val transactionRecord = transaction.transaction
        val chain = transactionRecord.assetId.chain
        val swapMetadata = getTransactionSwapMetadata(transactionRecord.type, transactionRecord.metadata)
        val swapProvider = swapMetadata?.provider?.toSwapProvider()
        if (transactionRecord.type == TransactionType.Swap && transactionRecord.state == TransactionState.InTransit && swapProvider == null) {
            return null
        }
        val request = TransactionStateRequest(
            id = transactionRecord.hash,
            senderAddress = transactionRecord.owner,
            createdAt = transactionRecord.createdAt,
            blockNumber = transactionRecord.blockNumber.toLongOrNull() ?: 0L,
        )
        val stateChanges = try {
            if (swapMetadata != null && swapProvider != null) {
                transactionStatusService.getSwapStatus(
                    chain,
                    TransactionSwapStateRequest(
                        transaction = request,
                        state = transactionRecord.state,
                        swapProvider = swapProvider,
                        destinationChain = swapMetadata.toAsset.chain,
                    ),
                )
            } else {
                transactionStatusService.getStatus(chain, request)
            }
        } catch (_: ServiceUnavailable) {
            return transaction.copy(transaction = transactionRecord.copy(updatedAt = System.currentTimeMillis()))
        }
        val newHash = stateChanges.hashChanges?.new
        val updatedTransaction = transaction.copy(
            transaction = transactionRecord.copy(
                id = newHash?.let { TransactionId(chain, it) } ?: transactionRecord.id,
                state = nextTransactionState(
                    oldState = transactionRecord.state,
                    newState = stateChanges.state,
                ),
                hash = newHash ?: transactionRecord.hash,
                fee = stateChanges.fee?.toString() ?: transactionRecord.fee,
                metadata = stateChanges.metadata ?: transactionRecord.metadata,
                confirmationEtaSeconds = stateChanges.confirmationEtaSeconds?.toLong() ?: transactionRecord.confirmationEtaSeconds,
            )
        )
        return if (updatedTransaction.transaction != transactionRecord) {
            updatedTransaction
        } else {
            null
        }
    }

    internal suspend fun storeTransactionUpdate(
        currentTransaction: DbTransactionExtended,
        updatedTransaction: DbTransactionExtended,
    ): DbTransactionExtended? {
        if (updatedTransaction.transaction.id == currentTransaction.transaction.id) {
            return updateTransaction(updatedTransaction)
        }

        val existingState = transactionsDao.getTransactionState(
            updatedTransaction.transaction.id,
            currentTransaction.transaction.walletId,
        )
        if (existingState == null) {
            transactionsDao.updateSwapMetadataTransactionId(
                oldTransactionId = currentTransaction.transaction.id.identifier,
                newTransactionId = updatedTransaction.transaction.id.identifier,
            )
            transactionsDao.updateTransactionId(
                oldId = currentTransaction.transaction.id,
                newId = updatedTransaction.transaction.id,
                walletId = currentTransaction.transaction.walletId,
                hash = updatedTransaction.transaction.hash,
            )
            return updateTransaction(updatedTransaction)
        }

        transactionsDao.deleteSwapMetadata(currentTransaction.transaction.id.identifier)
        transactionsDao.delete(
            currentTransaction.transaction.id,
            currentTransaction.transaction.walletId,
        )
        val nextState = updateExistingTransaction(
            placeholder = currentTransaction.transaction,
            updatedTransaction = updatedTransaction.transaction,
            existingState = existingState,
        )
        return updatedTransaction.copy(
            transaction = updatedTransaction.transaction.copy(
                state = nextState,
            ),
        )
    }

    internal fun transactionTimeout(transaction: DbTransaction): Long {
        val chain = transaction.assetId.chain
        val sourceTimeout = Config().getChainConfig(chain.string).transactionTimeout.toLong()
        if (transaction.state != TransactionState.InTransit) {
            return sourceTimeout
        }
        val destinationChain = getTransactionSwapMetadata(transaction.type, transaction.metadata)?.toAsset?.chain ?: chain
        if (destinationChain == chain) {
            return sourceTimeout
        }
        val destinationTimeout = Config().getChainConfig(destinationChain.string).transactionTimeout.toLong()
        return ((sourceTimeout + destinationTimeout) * 3).coerceAtLeast(DateUtils.DAY_IN_MILLIS)
    }

    private suspend fun updateTransaction(transaction: DbTransactionExtended): DbTransactionExtended? = withContext(Dispatchers.IO) {
        val transactionRecord = transaction.transaction.copy(updatedAt = System.currentTimeMillis())
        val updatedRows = transactionsDao.updateTransaction(
            id = transactionRecord.id,
            walletId = transactionRecord.walletId,
            state = transactionRecord.state,
            fee = transactionRecord.fee,
            metadata = transactionRecord.metadata,
            confirmationEtaSeconds = transactionRecord.confirmationEtaSeconds,
            updatedAt = transactionRecord.updatedAt,
        )
        if (updatedRows == 0) {
            return@withContext null
        }
        transactionsDao.addSwapMetadata(listOf(transactionRecord.toDTO()))
        transaction.copy(transaction = transactionRecord)
    }

    private fun updateExistingTransaction(
        placeholder: DbTransaction,
        updatedTransaction: DbTransaction,
        existingState: TransactionState,
    ): TransactionState {
        val nextState = nextTransactionState(
            oldState = existingState,
            newState = updatedTransaction.state,
        )
        if (nextState != existingState) {
            transactionsDao.updateState(updatedTransaction.id, updatedTransaction.walletId, nextState)
        }
        if (placeholder.fee != updatedTransaction.fee) {
            transactionsDao.updateFee(updatedTransaction.id, updatedTransaction.walletId, updatedTransaction.fee)
        }
        val metadata = updatedTransaction.metadata
        if (placeholder.metadata != metadata && metadata != null) {
            transactionsDao.updateMetadata(updatedTransaction.id, updatedTransaction.walletId, metadata)
            transactionsDao.addSwapMetadata(listOf(updatedTransaction.toDTO()))
        }
        return nextState
    }
}

internal fun nextTransactionState(oldState: TransactionState, newState: TransactionState): TransactionState {
    return if (oldState == TransactionState.Pending || newState.isCompleted()) newState else oldState
}
