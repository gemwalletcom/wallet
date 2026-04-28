package com.gemwallet.android.data.repositories.transactions

import android.text.format.DateUtils
import com.gemwallet.android.application.transactions.coordinators.GetChangedTransactions
import com.gemwallet.android.application.transactions.coordinators.GetPendingTransactionsCount
import com.gemwallet.android.blockchain.model.ServiceUnavailable
import com.gemwallet.android.blockchain.model.TransactionStateRequest
import com.gemwallet.android.blockchain.services.TransactionStatusService
import com.gemwallet.android.cases.transactions.ClearPendingTransactions
import com.gemwallet.android.cases.transactions.CreateTransaction
import com.gemwallet.android.cases.transactions.GetTransaction
import com.gemwallet.android.cases.transactions.SaveTransactions
import com.gemwallet.android.data.service.store.database.TransactionsDao
import com.gemwallet.android.data.service.store.database.entities.DbTransactionExtended
import com.gemwallet.android.data.service.store.database.entities.DbTxSwapMetadata
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.Fee
import com.gemwallet.android.model.Transaction
import com.gemwallet.android.model.TransactionChanges
import com.gemwallet.android.model.TransactionExtended
import com.gemwallet.android.serializer.jsonEncoder
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.TransactionDirection
import com.wallet.core.primitives.TransactionId
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.TransactionSwapMetadata
import com.wallet.core.primitives.TransactionType
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.mapNotNull
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.gemstone.Config
import java.math.BigInteger
import java.util.concurrent.ConcurrentHashMap

@OptIn(ExperimentalCoroutinesApi::class)
class TransactionsRepositoryImpl(
    private val transactionsDao: TransactionsDao,
    private val transactionStatusService: TransactionStatusService,
    private val scope: CoroutineScope = CoroutineScope(Dispatchers.IO),
)
: TransactionRepository,
    GetChangedTransactions,
    GetPendingTransactionsCount,
    GetTransaction,
    CreateTransaction,
    SaveTransactions,
    ClearPendingTransactions
{

    private val transactionsCheckDelay = 10 * DateUtils.SECOND_IN_MILLIS

    val changedTransactions = MutableStateFlow<List<TransactionExtended>>(emptyList())
    private val pendingTransactionJobs = ConcurrentHashMap<String, Job>()

    init {
        handlePendingTransactions()
    }

    override fun getPendingTransactionsCount(): Flow<Int?> {
        return transactionsDao.getTransactionsCount(TransactionState.Pending)
    }

    override fun getTransactions(): Flow<List<TransactionExtended>> {
        return transactionsDao.getExtendedTransactions()
            .mapNotNull { items -> items.toDTO() }
    }

    override fun getTransaction(transactionId: String): Flow<TransactionExtended?> {
        return transactionsDao.getExtendedTransaction(transactionId)
            .mapNotNull { it?.toDTO() }
            .flowOn(Dispatchers.IO)
    }

    override fun getChangedTransactions(): Flow<List<TransactionExtended>> = changedTransactions

    override suspend fun saveTransactions(walletId: String, transactions: List<Transaction>) = withContext(Dispatchers.IO) {
        transactionsDao.insert(transactions.toRecord(walletId))
        addSwapMetadata(transactions.filter { it.type == TransactionType.Swap })
    }

    private suspend fun updateTransaction(txs: List<DbTransactionExtended>) = withContext(Dispatchers.IO) {
        val data = txs.mapNotNull { it.toDTO()?.transaction?.toRecord(it.transaction.walletId) }
        transactionsDao.insert(data)
    }

    override suspend fun clearPending() {
        transactionsDao.deleteByState(TransactionState.Pending)
    }

    override suspend fun createTransaction(
        hash: String,
        walletId: String,
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

    private fun addSwapMetadata(txs: List<Transaction>) {
        val room = txs.filter { it.type == TransactionType.Swap && it.metadata != null }.mapNotNull {
            val txMetadata = it.metadata?.let { metadata ->
                jsonEncoder.decodeFromString<TransactionSwapMetadata>(metadata)
            } ?: return@mapNotNull null
            DbTxSwapMetadata(
                txId = it.id.identifier,
                fromAssetId = txMetadata.fromAsset.toIdentifier(),
                toAssetId = txMetadata.toAsset.toIdentifier(),
                fromAmount = txMetadata.fromValue,
                toAmount = txMetadata.toValue,
            )
        }
        transactionsDao.addSwapMetadata(room)
    }

    private fun handlePendingTransactions() {
        scope.launch {
            transactionsDao.getExtendedTransactions(TransactionState.Pending).collect { items ->
                items.forEach { item ->
                    if (!pendingTransactionJobs.containsKey(item.transaction.id)) {
                        val job = handlePendingTransaction(item)
                        pendingTransactionJobs.put(item.transaction.id, job)
                    }
                }
            }
        }
    }

    private fun handlePendingTransaction(tx: DbTransactionExtended) = scope.launch {
        val jobKey = tx.transaction.id
        try {
            var iteration = 0L
            var currentTx = tx
            val assetId = currentTx.transaction.assetId.toAssetId() ?: return@launch
            val chainConfig = Config().getChainConfig(assetId.chain.string)
            val delay = chainConfig.blockTime.toLong()
            val timeout = chainConfig.transactionTimeout.toLong()

            while (true) {
                transactionCheckDelay(delay, iteration)
                iteration++

                currentTx = checkTx(currentTx)?.let { newTx ->
                    if (newTx.transaction.id != currentTx.transaction.id) {
                        transactionsDao.delete(currentTx.transaction.id, currentTx.transaction.walletId)
                    }
                    updateTransaction(listOf(newTx))
                    newTx
                } ?: currentTx

                if (currentTx.transaction.createdAt < System.currentTimeMillis() - timeout) {
                    currentTx = currentTx.copy(transaction = currentTx.transaction.copy(state = TransactionState.Failed))
                    updateTransaction(listOf(currentTx))
                    break
                }
                if (currentTx.transaction.state != TransactionState.Pending) {
                    break
                }
            }
            currentTx.toDTO()?.let { changedTransactions.tryEmit(listOf(it)) }
        } finally {
            pendingTransactionJobs.remove(jobKey)
        }
    }

    private suspend fun transactionCheckDelay(delay: Long, iteration: Long) {
        val multiple = when (iteration) {
            0L -> 1.2
            1L -> 1.5
            2L -> 2.0
            3L -> 5.0
            else -> 10.0
        }
        val delay = (delay * multiple).toLong().takeIf { it < transactionsCheckDelay } ?: transactionsCheckDelay
        delay(delay)
    }

    private suspend fun checkTx(tx: DbTransactionExtended): DbTransactionExtended? {
        val assetId = tx.transaction.assetId.toAssetId() ?: return null
        val request = TransactionStateRequest(
            chain = assetId.chain,
            sender = tx.transaction.owner,
            hash = tx.transaction.hash,
            block = tx.transaction.blockNumber,
        )
        val state = try {
            transactionStatusService.getStatus(request) ?: TransactionChanges(tx.transaction.state)
        } catch (_: ServiceUnavailable) {
            return tx.copy(transaction = tx.transaction.copy(updatedAt = System.currentTimeMillis()))
        } catch (_: Throwable) {
            TransactionChanges(tx.transaction.state)
        }
        return if (state.state != tx.transaction.state) {
            val newTx = tx.copy(
                transaction = tx.transaction.copy(
                    id = if (state.hashChanges != null) {
                        "${assetId.chain.string}_${state.hashChanges!!.new}"
                    } else {
                        tx.transaction.id
                    },
                    state = state.state,
                    hash = if (state.hashChanges != null) state.hashChanges!!.new else tx.transaction.hash,
                )
            )
            when {
                state.fee != null -> newTx.copy(transaction = newTx.transaction.copy(fee = state.fee.toString()))
                else -> newTx
            }
        } else {
            null
        }
    }
}
