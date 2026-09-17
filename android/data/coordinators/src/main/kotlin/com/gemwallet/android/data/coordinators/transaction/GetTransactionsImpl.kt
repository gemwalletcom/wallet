package com.gemwallet.android.data.coordinators.transaction

import androidx.compose.runtime.Stable
import com.gemwallet.android.application.session.cases.GetCurrentWalletId
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
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import uniffi.gemstone.GemAmountSign
import uniffi.gemstone.transactionRow
import uniffi.gemstone.transactionRows
import uniffi.gemstone.GemTransactionRowSubtitle
import uniffi.gemstone.GemTransactionStatus
import uniffi.gemstone.GemTransactionRowValue
import uniffi.gemstone.GemTransactionTitle
import uniffi.gemstone.GemTransactionRow
import uniffi.gemstone.GemTransactionsServiceInterface
import uniffi.gemstone.GemValueStyle

private val usdFiatFormatter = CurrencyFormatter(type = CurrencyFormatter.Type.Fiat, currency = Currency.USD)
private val valueFormatter = ValueFormatter(style = GemValueStyle.SHORT)

class GetTransactionsImpl(
    private val getCurrentWalletId: GetCurrentWalletId,
    private val transactionStore: GemstoneTransactionStore,
    private val service: GemTransactionsServiceInterface,
) : GetTransactions {

    override fun getTransactions(
        filters: List<TransactionsRequestFilter>,
    ): Flow<List<TransactionDataAggregate>> = transactionStore.walletTransactions(getCurrentWalletId, filters)
        .aggregates()
        .flowOn(Dispatchers.IO)

    private fun Flow<List<TransactionExtended>>.aggregates(): Flow<List<TransactionDataAggregate>> = flow {
        val rows = TransactionRows()
        collect { emit(rows.aggregates(it)) }
    }
}

internal class TransactionRows {

    private var previous: Map<TransactionExtended, TransactionDataAggregate> = emptyMap()

    fun aggregates(items: List<TransactionExtended>): List<TransactionDataAggregate> {
        val reused = previous
        val missing = items.filterNot(reused::containsKey).distinct()
        val built = missing.zip(transactionRows(missing.map { it.toGem() })) { data, row ->
            data to TransactionDataAggregateImpl(row)
        }.toMap()
        val aggregates = items.mapNotNull { reused[it] ?: built[it] }
        previous = items.zip(aggregates).toMap()
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

    override val address: String = subtitle.address().orEmpty()

    private val coreValue: GemTransactionRowValue = row.value

    override val valueSign: GemAmountSign = (coreValue as? GemTransactionRowValue.Amount)?.amount?.sign ?: GemAmountSign.NONE

    override val value: String = coreValue.format().orEmpty()

    override val equivalentValue: String? = row.equivalentValue.format()

    override val nftImageUrl: String? = row.nftImageUrl

    override val type: TransactionType = row.transactionType.toPrimitives()

    override val direction: TransactionDirection = row.direction.toPrimitives()

    override val pnl: Double? = (coreValue as? GemTransactionRowValue.Pnl)?.value

    override val state: TransactionState = row.state.toPrimitives()

    override val createdAt: Long = row.createdAt
}

private fun GemTransactionRowSubtitle.address(): String? = when (this) {
    is GemTransactionRowSubtitle.ToAddress -> participant
    is GemTransactionRowSubtitle.FromAddress -> participant
    is GemTransactionRowSubtitle.ToResource,
    is GemTransactionRowSubtitle.FromResource,
    is GemTransactionRowSubtitle.Price,
    GemTransactionRowSubtitle.None -> null
}

private fun GemTransactionRowValue.format(): String? = when (this) {
    GemTransactionRowValue.None -> null
    is GemTransactionRowValue.AssetSymbol -> asset.symbol
    is GemTransactionRowValue.Amount -> amount.sign.format(valueFormatter.string(amount.value, amount.asset.toPrimitives()))
    is GemTransactionRowValue.Fiat -> usdFiatFormatter.string(value)
    is GemTransactionRowValue.Pnl -> PriceChangeFormatter(usdFiatFormatter).string(value)
}
