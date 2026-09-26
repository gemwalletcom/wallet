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
    private val observations = RecentFilters<Flow<List<GemTransactionRow>>>()

    init {
        getTransactions(activityFilters(emptyList(), emptyList()).toPrimitives()).launchIn(scope)
    }

    override fun getTransactions(filter: TransactionsFilter?): Flow<List<GemTransactionRow>> = observations.getOrPut(filter) {
        getCurrentWalletId()
            .flatMapLatest { walletId ->
                transactionsQuery(walletId, filter, GemConstants.transactionsListLimit).map { rowCache.rows(walletId, filter, it) }
            }
            .flowOn(Dispatchers.IO)
            .shareIn(scope, SharingStarted.WhileSubscribed(replayExpirationMillis = 0), replay = 1)
    }

    override fun stored(filter: TransactionsFilter?): List<GemTransactionRow> = getSession().value?.wallet?.id?.let { rowCache.stored(it, filter) }.orEmpty()
}

internal class TransactionRows {

    private class FilterRows(val walletId: WalletId, val items: Map<TransactionListItem, GemTransactionRow>, val rows: List<GemTransactionRow>)

    private val current = RecentFilters<FilterRows>()

    fun rows(walletId: WalletId, filter: TransactionsFilter?, items: List<TransactionListItem>): List<GemTransactionRow> = synchronized(this) {
        val reused = HashMap<TransactionListItem, GemTransactionRow>()
        current.values().forEach { reused.putAll(it.items) }
        val missing = items.filterNot(reused::containsKey).distinct()
        val built = missing.zip(transactionRows(missing.map { it.toGem() })).toMap()
        val rows = items.mapNotNull { reused[it] ?: built[it] }
        current.put(filter, FilterRows(walletId, items.zip(rows).toMap(), rows))
        rows
    }

    fun stored(walletId: WalletId, filter: TransactionsFilter?): List<GemTransactionRow> = synchronized(this) {
        current.get(filter)?.takeIf { it.walletId == walletId }?.rows.orEmpty()
    }
}

internal class RecentFilters<T>(private val limit: Int = LIMIT) {

    private val entries = LinkedHashMap<TransactionsFilter?, T>(limit, 0.75f, true)
    private val pinned = activityFilters(emptyList(), emptyList()).toPrimitives()

    @Synchronized
    fun get(filter: TransactionsFilter?): T? = entries[filter]

    @Synchronized
    fun getOrPut(filter: TransactionsFilter?, create: () -> T): T = entries[filter] ?: create().also { put(filter, it) }

    @Synchronized
    fun put(filter: TransactionsFilter?, value: T) {
        entries[filter] = value
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
