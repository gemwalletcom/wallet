package com.gemwallet.android.data.repositories.transactions

import android.util.Log
import com.gemwallet.android.application.transactions.coordinators.GetChangedTransactions
import com.gemwallet.android.application.transactions.coordinators.GetPendingTransactionsCount
import com.gemwallet.android.application.transactions.coordinators.TransactionsRequestFilter
import com.gemwallet.android.blockchain.model.ServiceUnavailable
import com.gemwallet.android.blockchain.services.TransactionStatusService
import com.gemwallet.android.cases.transactions.ClearPendingTransactions
import com.gemwallet.android.cases.transactions.CreateTransaction
import com.gemwallet.android.cases.transactions.SaveTransactions
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.service.store.database.TransactionsDao
import com.gemwallet.android.data.service.store.database.entities.DbTransactionExtended
import com.gemwallet.android.data.service.store.database.entities.DbTxSwapMetadata
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.ext.getTransactionSwapMetadata
import com.gemwallet.android.ext.isCompleted
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.toSwapProvider
import com.gemwallet.android.model.Fee
import com.gemwallet.android.model.TransactionExtended
import com.gemwallet.android.serializer.jsonEncoder
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Transaction
import com.wallet.core.primitives.TransactionDirection
import com.wallet.core.primitives.TransactionId
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.TransactionStateRequest
import com.wallet.core.primitives.TransactionSwapMetadata
import com.wallet.core.primitives.TransactionSwapStateRequest
import com.wallet.core.primitives.TransactionType
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapNotNull
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.gemstone.transactionStateConfig
import java.math.BigInteger
import java.util.concurrent.ConcurrentHashMap

private val pollingTransactionStates = listOf(TransactionState.Pending, TransactionState.InTransit)
private const val TAG = "TransactionsRepository"

@OptIn(ExperimentalCoroutinesApi::class)
class TransactionsRepositoryImpl(
    private val sessionRepository: SessionRepository,
    private val transactionsDao: TransactionsDao,
    private val transactionStatusService: TransactionStatusService,
    private val scope: CoroutineScope = CoroutineScope(Dispatchers.IO),
) : TransactionRepository,
    GetChangedTransactions,
    GetPendingTransactionsCount,
    CreateTransaction,
    SaveTransactions,
    ClearPendingTransactions {

    val changedTransactions = MutableStateFlow<List<TransactionExtended>>(emptyList())
    private val pollingTransactionJobs = ConcurrentHashMap<TransactionId, Job>()

    private fun currentWalletId(): Flow<WalletId> = sessionRepository.session()
        .filterNotNull()
        .map { it.wallet.id }
        .distinctUntilChanged()

    init {
        observePollingTransactions()
    }

    override fun getPendingTransactionsCount(): Flow<Int?> {
        return currentWalletId().flatMapLatest { walletId ->
            transactionsDao.getTransactionsCount(walletId, pollingTransactionStates)
        }
    }

    override fun getTransactions(filters: List<TransactionsRequestFilter>): Flow<List<TransactionExtended>> {
        return currentWalletId().flatMapLatest { walletId ->
            transactionsDao.getExtendedTransactions(walletId, filters)
        }.mapNotNull { items -> items.toDTO() }
    }

    override fun getTransaction(transactionId: TransactionId): Flow<TransactionExtended?> {
        return currentWalletId().flatMapLatest { walletId ->
            transactionsDao.getExtendedTransaction(walletId, transactionId)
        }.mapNotNull { it?.toDTO() }
            .flowOn(Dispatchers.IO)
    }

    override fun getChangedTransactions(): Flow<List<TransactionExtended>> = changedTransactions

    override suspend fun saveTransactions(walletId: WalletId, transactions: List<Transaction>) = withContext(Dispatchers.IO) {
        transactionsDao.syncTransactions(transactions.toRecord(walletId))
        addSwapMetadata(transactions)
    }

    private suspend fun updateTransactions(transactions: List<DbTransactionExtended>) = withContext(Dispatchers.IO) {
        val updatedAt = System.currentTimeMillis()
        val records = transactions.map { it.transaction.copy(updatedAt = updatedAt) }
        transactionsDao.insert(records)
        addSwapMetadata(records.map { it.toDTO() })
    }

    override suspend fun clearPending() {
        transactionsDao.deleteByState(TransactionState.Pending)
    }

    override suspend fun createTransaction(
        hash: String,
        walletId: WalletId,
        assetId: AssetId,
        owner: Account,
        to: String,
        state: TransactionState,
        fee: Fee,
        amount: BigInteger,
        memo: String?,
        type: TransactionType,
        metadata: String?,
        direction: TransactionDirection,
        blockNumber: String,
    ): Transaction = withContext(Dispatchers.IO) {
        val transaction = Transaction(
            id = TransactionId(assetId.chain, hash),
            assetId = assetId,
            feeAssetId = fee.feeAssetId,
            from = owner.address,
            to = to,
            type = type,
            state = state,
            blockNumber = blockNumber,
            sequence = "", // Nonce
            fee = fee.amount.toString(),
            value = amount.toString(),
            memo = if (type == TransactionType.Swap) "" else memo,
            direction = direction,
            metadata = metadata,
            utxoInputs = emptyList(),
            utxoOutputs = emptyList(),
            createdAt = System.currentTimeMillis(),
        )
        transactionsDao.insert(listOf(transaction.toRecord(walletId)))
        addSwapMetadata(listOf(transaction))
        transaction
    }

    private fun addSwapMetadata(transactions: List<Transaction>) {
        val swapMetadataRecords = transactions.mapNotNull { transaction ->
            if (transaction.type != TransactionType.Swap) {
                return@mapNotNull null
            }
            val metadata = transaction.metadata ?: return@mapNotNull null
            val swapMetadata = jsonEncoder.decodeFromString<TransactionSwapMetadata>(metadata)
            DbTxSwapMetadata(
                txId = transaction.id.identifier,
                fromAssetId = swapMetadata.fromAsset.toIdentifier(),
                toAssetId = swapMetadata.toAsset.toIdentifier(),
                fromAmount = swapMetadata.fromValue,
                toAmount = swapMetadata.toValue,
            )
        }
        transactionsDao.addSwapMetadata(swapMetadataRecords)
    }

    private fun observePollingTransactions() {
        scope.launch {
            currentWalletId().flatMapLatest { walletId ->
                transactionsDao.getExtendedTransactions(
                    walletId,
                    listOf(TransactionsRequestFilter.States(pollingTransactionStates)),
                )
            }.collect { items ->
                items.forEach { item ->
                    if (!pollingTransactionJobs.containsKey(item.transaction.id)) {
                        val job = pollTransactionStatus(item)
                        pollingTransactionJobs.put(item.transaction.id, job)
                    }
                }
            }
        }
    }

    private fun pollTransactionStatus(transaction: DbTransactionExtended) = scope.launch {
        val jobKeys = mutableSetOf(transaction.transaction.id)
        try {
            var currentTransaction = transaction
            val jobConfig = transactionStateConfig(currentTransaction.transaction.assetId.chain.string)
            var pollingDelay = jobConfig.initialIntervalMs

            while (true) {
                delay(pollingDelay.toLong())
                pollingDelay = jobConfig.nextIntervalMs(pollingDelay)

                currentTransaction = storedTransaction(currentTransaction) ?: break

                checkTransaction(currentTransaction)?.let { updatedTransaction ->
                    if (updatedTransaction.transaction.id != currentTransaction.transaction.id) {
                        coroutineContext[Job]?.let { runningJob ->
                            pollingTransactionJobs[updatedTransaction.transaction.id] = runningJob
                            jobKeys.add(updatedTransaction.transaction.id)
                        }
                    }
                    currentTransaction = storeTransactionUpdate(
                        currentTransaction = currentTransaction,
                        updatedTransaction = updatedTransaction,
                    )
                }

                if (currentTransaction.transaction.state.isCompleted()) {
                    Log.d(
                        TAG,
                        "transaction status complete: id=${currentTransaction.transaction.id.identifier}, state=${currentTransaction.transaction.state}, status=complete",
                    )
                    break
                }
                Log.d(
                    TAG,
                    "transaction status pending: id=${currentTransaction.transaction.id.identifier}, state=${currentTransaction.transaction.state}, next check = ${pollingDelay}ms",
                )
            }
            currentTransaction.toDTO()?.let { changedTransactions.tryEmit(listOf(it)) }
        } finally {
            jobKeys.forEach { pollingTransactionJobs.remove(it) }
        }
    }

    private suspend fun storedTransaction(transaction: DbTransactionExtended): DbTransactionExtended? =
        transactionsDao.getExtendedTransaction(
            transaction.transaction.walletId,
            transaction.transaction.id,
        ).first()

    private suspend fun checkTransaction(transaction: DbTransactionExtended): DbTransactionExtended? {
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
            )
        )
        return if (updatedTransaction.transaction != transactionRecord) {
            updatedTransaction
        } else {
            null
        }
    }

    private suspend fun storeTransactionUpdate(
        currentTransaction: DbTransactionExtended,
        updatedTransaction: DbTransactionExtended,
    ): DbTransactionExtended {
        if (updatedTransaction.transaction.id == currentTransaction.transaction.id) {
            updateTransactions(listOf(updatedTransaction))
            return updatedTransaction
        }

        val walletId = currentTransaction.transaction.walletId
        transactionsDao.deleteSwapMetadata(updatedTransaction.transaction.id.identifier)
        transactionsDao.delete(updatedTransaction.transaction.id, walletId)
        transactionsDao.updateSwapMetadataTransactionId(
            oldTransactionId = currentTransaction.transaction.id.identifier,
            newTransactionId = updatedTransaction.transaction.id.identifier,
        )
        transactionsDao.updateTransactionId(
            oldId = currentTransaction.transaction.id,
            newId = updatedTransaction.transaction.id,
            walletId = walletId,
            hash = updatedTransaction.transaction.hash,
        )
        updateTransactions(listOf(updatedTransaction))
        return updatedTransaction
    }
}

internal fun nextTransactionState(oldState: TransactionState, newState: TransactionState): TransactionState {
    return if (oldState == TransactionState.Pending || newState.isCompleted()) newState else oldState
}
