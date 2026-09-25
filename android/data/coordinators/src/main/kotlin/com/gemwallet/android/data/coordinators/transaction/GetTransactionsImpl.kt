package com.gemwallet.android.data.coordinators.transaction

import com.gemwallet.android.application.session.cases.GetCurrentWalletId
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.transactions.cases.GetTransactions
import com.gemwallet.android.application.transactions.cases.TransactionsRequestFilter
import com.gemwallet.android.data.services.gemstone.stores.GemstoneTransactionStore
import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.TransactionListItem
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
import uniffi.gemstone.transactionRows

@OptIn(ExperimentalCoroutinesApi::class)
class GetTransactionsImpl(private val getSession: GetSession, private val getCurrentWalletId: GetCurrentWalletId, private val transactionStore: GemstoneTransactionStore, private val scope: CoroutineScope = CoroutineScope(Dispatchers.IO)) :
    GetTransactions {

    private val rowCache = TransactionRows()
    private val observations = RecentFilters<Flow<List<GemTransactionRow>>>()

    init {
        getTransactions(TransactionsRequestFilter.activityDefaults()).launchIn(scope)
    }

    override fun getTransactions(filters: List<TransactionsRequestFilter>): Flow<List<GemTransactionRow>> = observations.getOrPut(filters) {
        getCurrentWalletId()
            .flatMapLatest { walletId ->
                transactionStore.observeTransactions(walletId, filters).map { rowCache.rows(walletId, filters, it) }
            }
            .flowOn(Dispatchers.IO)
            .shareIn(scope, SharingStarted.WhileSubscribed(replayExpirationMillis = 0), replay = 1)
    }

    override fun stored(filters: List<TransactionsRequestFilter>): List<GemTransactionRow> = getSession().value?.wallet?.id?.let { rowCache.stored(it, filters) }.orEmpty()
}

internal class TransactionRows {

    private class FilterRows(val walletId: WalletId, val items: Map<TransactionListItem, GemTransactionRow>, val rows: List<GemTransactionRow>)

    private val current = RecentFilters<FilterRows>()

    fun rows(walletId: WalletId, filters: List<TransactionsRequestFilter>, items: List<TransactionListItem>): List<GemTransactionRow> = synchronized(this) {
        val reused = HashMap<TransactionListItem, GemTransactionRow>()
        current.values().forEach { reused.putAll(it.items) }
        val missing = items.filterNot(reused::containsKey).distinct()
        val built = missing.zip(transactionRows(missing.map { it.toGem() })).toMap()
        val rows = items.mapNotNull { reused[it] ?: built[it] }
        current.put(filters, FilterRows(walletId, items.zip(rows).toMap(), rows))
        rows
    }

    fun stored(walletId: WalletId, filters: List<TransactionsRequestFilter>): List<GemTransactionRow> = synchronized(this) {
        current.get(filters)?.takeIf { it.walletId == walletId }?.rows.orEmpty()
    }
}

internal class RecentFilters<T>(private val limit: Int = LIMIT) {

    private val entries = LinkedHashMap<List<TransactionsRequestFilter>, T>(limit, 0.75f, true)
    private val pinned = TransactionsRequestFilter.activityDefaults()

    @Synchronized
    fun get(filters: List<TransactionsRequestFilter>): T? = entries[filters]

    @Synchronized
    fun getOrPut(filters: List<TransactionsRequestFilter>, create: () -> T): T = entries[filters] ?: create().also { put(filters, it) }

    @Synchronized
    fun put(filters: List<TransactionsRequestFilter>, value: T) {
        entries[filters] = value
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
