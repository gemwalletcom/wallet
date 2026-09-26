package com.gemwallet.android.data.coordinators.transaction

import com.gemwallet.android.application.session.cases.GetCurrentWalletId
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.transactions.cases.GetTransactions
import com.gemwallet.android.data.services.store.queries.TransactionsQuery
import com.gemwallet.android.ext.GemConstants
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.wallet.core.primitives.TransactionListItem
import com.wallet.core.primitives.TransactionsFilter
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.launchIn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.shareIn
import uniffi.gemstone.GemTransactionRow
import uniffi.gemstone.activityFilters
import uniffi.gemstone.transactionRows

@OptIn(ExperimentalCoroutinesApi::class)
class GetTransactionsImpl(private val getSession: GetSession, private val getCurrentWalletId: GetCurrentWalletId, private val transactionsQuery: TransactionsQuery, private val scope: CoroutineScope = CoroutineScope(Dispatchers.IO)) :
    GetTransactions {

    private val rowCache = TransactionRows()
    private val observations = RecentRequests<Flow<List<GemTransactionRow>>>()

    init {
        getTransactions(activityFilters(emptyList(), emptyList()).toPrimitives(), GemConstants.transactionsListLimit).launchIn(scope)
    }

    override fun getTransactions(filter: TransactionsFilter?, limit: Int): Flow<List<GemTransactionRow>> = TransactionsRequest(filter, limit).let { request ->
        observations.getOrPut(request) {
            getCurrentWalletId()
                .flatMapLatest { walletId ->
                    transactionsQuery(walletId, filter, limit).map { rowCache.rows(walletId, request, it) }
                }
                .flowOn(Dispatchers.IO)
                .shareIn(scope, SharingStarted.WhileSubscribed(replayExpirationMillis = 0), replay = 1)
        }
    }

    override fun stored(filter: TransactionsFilter?, limit: Int): List<GemTransactionRow> = getSession().value?.wallet?.id?.let { rowCache.stored(it, TransactionsRequest(filter, limit)) }.orEmpty()
}

internal data class TransactionsRequest(val filter: TransactionsFilter?, val limit: Int)

internal class TransactionRows {

    private class FilterRows(val walletId: WalletId, val items: Map<TransactionListItem, GemTransactionRow>, val rows: List<GemTransactionRow>)

    private val current = RecentRequests<FilterRows>()

    fun rows(walletId: WalletId, request: TransactionsRequest, items: List<TransactionListItem>): List<GemTransactionRow> = synchronized(this) {
        val reused = HashMap<TransactionListItem, GemTransactionRow>()
        current.values().forEach { reused.putAll(it.items) }
        val missing = items.filterNot(reused::containsKey).distinct()
        val built = missing.zip(transactionRows(missing.map { it.toGem() })).toMap()
        val rows = items.mapNotNull { reused[it] ?: built[it] }
        current.put(request, FilterRows(walletId, items.zip(rows).toMap(), rows))
        rows
    }

    fun stored(walletId: WalletId, request: TransactionsRequest): List<GemTransactionRow> = synchronized(this) {
        current.get(request)?.takeIf { it.walletId == walletId }?.rows.orEmpty()
    }
}

internal class RecentRequests<T>(private val limit: Int = LIMIT) {

    private val entries = LinkedHashMap<TransactionsRequest, T>(limit, 0.75f, true)
    private val pinned = TransactionsRequest(activityFilters(emptyList(), emptyList()).toPrimitives(), GemConstants.transactionsListLimit)

    @Synchronized
    fun get(request: TransactionsRequest): T? = entries[request]

    @Synchronized
    fun getOrPut(request: TransactionsRequest, create: () -> T): T = entries[request] ?: create().also { put(request, it) }

    @Synchronized
    fun put(request: TransactionsRequest, value: T) {
        entries[request] = value
        while (entries.size > limit) {
            entries.keys.firstOrNull { it != pinned }?.let(entries::remove) ?: break
        }
    }

    @Synchronized
    fun values(): List<T> = entries.values.toList()

    companion object {
        const val LIMIT = 8
    }
}
