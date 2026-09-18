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
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.model.PriceChangeFormatter
import com.gemwallet.android.model.ValueFormatter
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.TransactionDirection
import com.wallet.core.primitives.TransactionExtended
import com.wallet.core.primitives.TransactionId
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.TransactionType
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.launchIn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onEach
import uniffi.gemstone.transactionRow
import uniffi.gemstone.transactionRows
import uniffi.gemstone.GemTransactionRowSubtitle
import uniffi.gemstone.GemTransactionStatus
import uniffi.gemstone.GemTransactionRowValue
import uniffi.gemstone.GemTransactionTitle
import uniffi.gemstone.GemTransactionRow
import uniffi.gemstone.GemTransactionsServiceInterface
import uniffi.gemstone.GemValueStyle
import java.util.concurrent.ConcurrentHashMap
import uniffi.gemstone.GemValueTone

private val usdFiatFormatter = CurrencyFormatter(type = CurrencyFormatter.Type.Fiat, currency = Currency.USD)
private val valueFormatter = ValueFormatter(style = GemValueStyle.SHORT)

@OptIn(ExperimentalCoroutinesApi::class)
class GetTransactionsImpl(
    private val getSession: GetSession,
    private val getCurrentWalletId: GetCurrentWalletId,
    private val transactionStore: GemstoneTransactionStore,
    private val service: GemTransactionsServiceInterface,
    scope: CoroutineScope = CoroutineScope(Dispatchers.IO),
) : GetTransactions {

    private val rows = TransactionRows()
    private val stored = ConcurrentHashMap<List<TransactionsRequestFilter>, WalletTransactionRows>()

    init {
        getTransactions(TransactionsRequestFilter.activityDefaults()).launchIn(scope)
    }

    override fun getTransactions(
        filters: List<TransactionsRequestFilter>,
    ): Flow<List<TransactionDataAggregate>> = getCurrentWalletId()
        .flatMapLatest { walletId ->
            transactionStore.observeTransactions(walletId, filters)
                .map { WalletTransactionRows(walletId, rows.aggregates(filters, it)) }
        }
        .onEach { stored[filters] = it }
        .map { it.rows }
        .flowOn(Dispatchers.IO)

    override fun stored(filters: List<TransactionsRequestFilter>): List<TransactionDataAggregate> =
        stored[filters]?.takeIf { it.walletId == getSession().value?.wallet?.id }?.rows.orEmpty()
}

private class WalletTransactionRows(
    val walletId: WalletId,
    val rows: List<TransactionDataAggregate>,
)

internal class TransactionRows {

    private val current = HashMap<List<TransactionsRequestFilter>, Map<TransactionExtended, TransactionDataAggregate>>()

    @Synchronized
    fun aggregates(filters: List<TransactionsRequestFilter>, items: List<TransactionExtended>): List<TransactionDataAggregate> {
        val reused = HashMap<TransactionExtended, TransactionDataAggregate>()
        current.values.forEach(reused::putAll)
        val missing = items.filterNot(reused::containsKey).distinct()
        val built = missing.zip(transactionRows(missing.map { it.toGem() })) { data, row ->
            data to TransactionDataAggregateImpl(row)
        }.toMap()
        val aggregates = items.mapNotNull { reused[it] ?: built[it] }
        current[filters] = items.zip(aggregates).toMap()
        return aggregates
    }
}

@Stable
class TransactionDataAggregateImpl(
    private val row: GemTransactionRow,
) : TransactionDataAggregate {

    override val id: TransactionId = TransactionId(row.id)

    override val asset: Asset = row.asset.toPrimitives()

    override val status: GemTransactionStatus = row.status

    override val title: GemTransactionTitle = row.title

    override val subtitle: GemTransactionRowSubtitle = row.subtitle

    private val coreValue: GemTransactionRowValue = row.value

    override val valueTone: GemValueTone = row.valueTone

    override val value: String = coreValue.format().orEmpty()

    override val equivalentValue: String? = row.equivalentValue.format()

    override val nftImageUrl: String? = row.nftImageUrl

    override val type: TransactionType = row.transactionType.toPrimitives()

    override val direction: TransactionDirection = row.direction.toPrimitives()

    override val state: TransactionState = row.state.toPrimitives()

    override val createdAt: Long = row.createdAt
}

private fun GemTransactionRowValue.format(): String? = when (this) {
    GemTransactionRowValue.None -> null
    is GemTransactionRowValue.AssetSymbol -> asset.symbol
    is GemTransactionRowValue.Amount -> amount.sign.format(valueFormatter.string(amount.value, amount.asset.toPrimitives()))
    is GemTransactionRowValue.Fiat -> usdFiatFormatter.string(value)
    is GemTransactionRowValue.Pnl -> PriceChangeFormatter(usdFiatFormatter).string(value)
}
