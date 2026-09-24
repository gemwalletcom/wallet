package com.gemwallet.android.data.coordinators.transaction

import androidx.compose.runtime.Stable
import com.gemwallet.android.application.session.cases.GetCurrentWalletId
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.transactions.cases.GetTransactions
import com.gemwallet.android.application.transactions.cases.TransactionsRequestFilter
import com.gemwallet.android.data.services.gemstone.stores.GemstoneTransactionStore
import com.gemwallet.android.domains.transaction.aggregates.TransactionDataAggregate
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.text
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.TransactionExtended
import com.wallet.core.primitives.TransactionId
import com.wallet.core.primitives.TransactionState
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
import uniffi.gemstone.GemAssetIcon
import uniffi.gemstone.GemTransactionBadge
import uniffi.gemstone.GemTransactionRow
import uniffi.gemstone.GemTransactionRowSubtitle
import uniffi.gemstone.GemTransactionRowValue
import uniffi.gemstone.GemTransactionStatus
import uniffi.gemstone.GemTransactionTitle
import uniffi.gemstone.GemValueTone
import uniffi.gemstone.transactionRows

@OptIn(ExperimentalCoroutinesApi::class)
class GetTransactionsImpl(private val getSession: GetSession, private val getCurrentWalletId: GetCurrentWalletId, private val transactionStore: GemstoneTransactionStore, private val scope: CoroutineScope = CoroutineScope(Dispatchers.IO)) :
    GetTransactions {

    private val rows = TransactionRows()
    private val observations = RecentFilters<Flow<List<TransactionDataAggregate>>>()

    init {
        getTransactions(TransactionsRequestFilter.activityDefaults()).launchIn(scope)
    }

    override fun getTransactions(filters: List<TransactionsRequestFilter>): Flow<List<TransactionDataAggregate>> = observations.getOrPut(filters) {
        getCurrentWalletId()
            .flatMapLatest { walletId ->
                transactionStore.observeTransactions(walletId, filters).map { rows.aggregates(walletId, filters, it) }
            }
            .flowOn(Dispatchers.IO)
            .shareIn(scope, SharingStarted.WhileSubscribed(replayExpirationMillis = 0), replay = 1)
    }

    override fun stored(filters: List<TransactionsRequestFilter>): List<TransactionDataAggregate> = getSession().value?.wallet?.id?.let { rows.stored(it, filters) }.orEmpty()
}

internal class TransactionRows {

    private class FilterRows(val walletId: WalletId, val items: Map<TransactionExtended, TransactionDataAggregate>, val rows: List<TransactionDataAggregate>)

    private val current = RecentFilters<FilterRows>()

    fun aggregates(walletId: WalletId, filters: List<TransactionsRequestFilter>, items: List<TransactionExtended>): List<TransactionDataAggregate> = synchronized(this) {
        val reused = HashMap<TransactionExtended, TransactionDataAggregate>()
        current.values().forEach { reused.putAll(it.items) }
        val missing = items.filterNot(reused::containsKey).distinct()
        val built = missing.zip(transactionRows(missing.map { it.toGem() })) { data, row ->
            data to TransactionDataAggregateImpl(row)
        }.toMap()
        val aggregates = items.mapNotNull { reused[it] ?: built[it] }
        current.put(filters, FilterRows(walletId, items.zip(aggregates).toMap(), aggregates))
        aggregates
    }

    fun stored(walletId: WalletId, filters: List<TransactionsRequestFilter>): List<TransactionDataAggregate> = synchronized(this) {
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

@Stable
class TransactionDataAggregateImpl(private val row: GemTransactionRow) : TransactionDataAggregate {

    override val id: TransactionId = TransactionId(row.id)

    override val asset: Asset = row.asset.toPrimitives()

    override val icon: GemAssetIcon = row.icon

    override val status: GemTransactionStatus = row.status

    override val title: GemTransactionTitle = row.title

    override val subtitle: GemTransactionRowSubtitle = row.subtitle

    private val coreValue: GemTransactionRowValue = row.value

    override val valueTone: GemValueTone = row.valueTone

    override val value: String = coreValue.format().orEmpty()

    override val equivalentValue: String? = row.equivalentValue.format()

    override val nftImageUrl: String? = row.nftImageUrl

    override val badge: GemTransactionBadge = row.badge

    override val state: TransactionState = row.state.toPrimitives()

    override val createdAt: Long = row.createdAt
}

private fun GemTransactionRowValue.format(): String? = when (this) {
    GemTransactionRowValue.None -> null
    is GemTransactionRowValue.AssetSymbol -> asset.symbol
    is GemTransactionRowValue.Number -> number.text()
}
