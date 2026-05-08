package com.gemwallet.android.data.repositories.transactions

import android.text.format.DateUtils
import com.gemwallet.android.application.transactions.coordinators.GetChangedTransactions
import com.gemwallet.android.application.transactions.coordinators.GetPendingTransactionsCount
import com.gemwallet.android.application.transactions.coordinators.TransactionsRequestFilter
import com.gemwallet.android.blockchain.model.ServiceUnavailable
import com.gemwallet.android.blockchain.model.TransactionStateRequest
import com.gemwallet.android.blockchain.services.TransactionStatusService
import com.gemwallet.android.cases.transactions.ClearPendingTransactions
import com.gemwallet.android.cases.transactions.CreateTransaction
import com.gemwallet.android.cases.transactions.GetTransaction
import com.gemwallet.android.cases.transactions.SaveTransactions
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.service.store.database.TransactionsDao
import com.gemwallet.android.data.service.store.database.entities.DbTransactionExtended
import com.gemwallet.android.data.service.store.database.entities.DbTxSwapMetadata
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.walletId
import com.gemwallet.android.model.Fee
import com.wallet.core.primitives.Transaction
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
import java.math.BigInteger
import java.util.concurrent.ConcurrentHashMap

@OptIn(ExperimentalCoroutinesApi::class)
class TransactionsRepositoryImpl(
    private val sessionRepository: SessionRepository,
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
    private val pendingTransactionJobs = ConcurrentHashMap<TransactionId, Job>()

    private fun currentWalletId(): Flow<WalletId> = sessionRepository.session()
        .filterNotNull()
        .map { it.wallet.walletId }
        .distinctUntilChanged()

    init {
        handlePendingTransactions()
    }

    override fun getPendingTransactionsCount(): Flow<Int?> {
        return currentWalletId().flatMapLatest { walletId ->
            transactionsDao.getTransactionsCount(walletId, TransactionState.Pending)
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
        addSwapMetadata(transactions.filter { it.type == TransactionType.Swap })
    }

    private suspend fun updateTransaction(transactions: List<DbTransactionExtended>) = withContext(Dispatchers.IO) {
        val data = transactions.mapNotNull { it.toDTO()?.transaction?.toRecord(it.transaction.walletId) }
        transactionsDao.insert(data)
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
            currentWalletId().flatMapLatest { walletId ->
                transactionsDao.getExtendedTransactions(
                    walletId,
                    listOf(TransactionsRequestFilter.State(TransactionState.Pending)),
                )
            }.collect { items ->
                items.forEach { item ->
                    if (!pendingTransactionJobs.containsKey(item.transaction.id)) {
                        val job = handlePendingTransaction(item)
                        pendingTransactionJobs.put(item.transaction.id, job)
                    }
                }
            }
        }
    }

    private fun handlePendingTransaction(transaction: DbTransactionExtended) = scope.launch {
        val jobKey = transaction.transaction.id
        try {
            var iteration = 0L
            var currentTransaction = transaction
            val chainConfig = Config().getChainConfig(currentTransaction.transaction.assetId.chain.string)
            val delay = chainConfig.blockTime.toLong()
            val timeout = chainConfig.transactionTimeout.toLong()

            while (true) {
                transactionCheckDelay(delay, iteration)
                iteration++

                currentTransaction = checkTx(currentTransaction)?.let { newTransaction ->
                    if (newTransaction.transaction.id != currentTransaction.transaction.id) {
                        transactionsDao.delete(
                            currentTransaction.transaction.id,
                            currentTransaction.transaction.walletId,
                        )
                    }
                    updateTransaction(listOf(newTransaction))
                    newTransaction
                } ?: currentTransaction

                if (currentTransaction.transaction.createdAt < System.currentTimeMillis() - timeout) {
                    currentTransaction = currentTransaction.copy(transaction = currentTransaction.transaction.copy(state = TransactionState.Failed))
                    updateTransaction(listOf(currentTransaction))
                    break
                }
                if (currentTransaction.transaction.state != TransactionState.Pending) {
                    break
                }
            }
            currentTransaction.toDTO()?.let { changedTransactions.tryEmit(listOf(it)) }
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

    private suspend fun checkTx(transaction: DbTransactionExtended): DbTransactionExtended? {
        val request = TransactionStateRequest(
            chain = transaction.transaction.assetId.chain,
            sender = transaction.transaction.owner,
            hash = transaction.transaction.hash,
            block = transaction.transaction.blockNumber,
        )
        val state = try {
            transactionStatusService.getStatus(request) ?: TransactionChanges(transaction.transaction.state)
        } catch (_: ServiceUnavailable) {
            return transaction.copy(transaction = transaction.transaction.copy(updatedAt = System.currentTimeMillis()))
        } catch (_: Throwable) {
            TransactionChanges(transaction.transaction.state)
        }
        return if (state.state != transaction.transaction.state) {
            val newTransaction = transaction.copy(
                transaction = transaction.transaction.copy(
                    id = if (state.hashChanges != null) {
                        TransactionId(transaction.transaction.assetId.chain, state.hashChanges!!.new)
                    } else {
                        transaction.transaction.id
                    },
                    state = state.state,
                    hash = if (state.hashChanges != null) state.hashChanges!!.new else transaction.transaction.hash,
                )
            )
            when {
                state.fee != null -> newTransaction.copy(transaction = newTransaction.transaction.copy(fee = state.fee.toString()))
                else -> newTransaction
            }
        } else {
            null
        }
    }
}
