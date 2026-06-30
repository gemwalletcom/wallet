package com.gemwallet.android.data.repositories.transactions

import android.text.format.DateUtils
import android.util.Log
import com.gemwallet.android.application.transactions.coordinators.GetChangedTransactions
import com.gemwallet.android.application.transactions.coordinators.GetPendingTransactionsCount
import com.gemwallet.android.application.transactions.coordinators.TransactionsRequestFilter
import com.gemwallet.android.blockchain.model.ServiceUnavailable
import com.gemwallet.android.blockchain.services.TransactionStatusService
import com.gemwallet.android.cases.transactions.ClearPendingTransactions
import com.gemwallet.android.cases.transactions.CreateTransaction
import com.gemwallet.android.cases.transactions.GetTransaction
import com.gemwallet.android.cases.transactions.SaveTransactions
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.service.store.database.TransactionsDao
import com.gemwallet.android.data.service.store.database.entities.DbTransaction
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
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapNotNull
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.gemstone.Config
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
    GetTransaction,
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
        transactionsDao.insert(transactions.toRecord(walletId))
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

                val hasTimedOut = !currentTransaction.transaction.state.isCompleted() &&
                    currentTransaction.transaction.createdAt < System.currentTimeMillis() - transactionTimeout(currentTransaction.transaction)
                if (hasTimedOut) {
                    currentTransaction = currentTransaction.copy(transaction = currentTransaction.transaction.copy(state = TransactionState.Failed))
                    updateTransactions(listOf(currentTransaction))
                    break
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

    private fun transactionTimeout(transaction: DbTransaction): Long {
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
            updateTransactions(listOf(updatedTransaction))
            return updatedTransaction
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
            )
        )
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
            addSwapMetadata(listOf(updatedTransaction.toDTO()))
        }
        return nextState
    }
}

internal fun nextTransactionState(oldState: TransactionState, newState: TransactionState): TransactionState {
    return if (oldState == TransactionState.Pending || newState.isCompleted()) newState else oldState
}
