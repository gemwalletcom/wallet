package com.gemwallet.android.data.coordinators.transaction

import androidx.compose.runtime.Stable
import com.gemwallet.android.application.session.cases.GetCurrentWalletId
import com.gemwallet.android.application.transactions.cases.GetTransactions
import com.gemwallet.android.application.transactions.cases.TransactionsRequestFilter
import com.gemwallet.android.data.services.gemstone.stores.GemstoneTransactionStore
import com.gemwallet.android.domains.transaction.aggregates.TransactionDataAggregate
import com.gemwallet.android.domains.transaction.format
import com.gemwallet.android.ext.AddressFormatter
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.model.PriceChangeFormatter
import com.gemwallet.android.model.ValueFormatter
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.TransactionDirection
import com.wallet.core.primitives.TransactionExtended
import com.wallet.core.primitives.TransactionId
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.TransactionType
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import uniffi.gemstone.GemAddressService
import uniffi.gemstone.GemAmountSign
import uniffi.gemstone.GemTransactionRow
import uniffi.gemstone.GemTransactionRowSubtitle
import uniffi.gemstone.GemTransactionRowValue
import uniffi.gemstone.GemTransactionTitle

private val usdFiatFormatter = CurrencyFormatter(type = CurrencyFormatter.Type.Fiat, currency = Currency.USD)
private val valueFormatter = ValueFormatter(style = ValueFormatter.Style.Short)

class GetTransactionsImpl(
    private val getCurrentWalletId: GetCurrentWalletId,
    private val transactionStore: GemstoneTransactionStore,
    private val addressService: GemAddressService,
    scope: CoroutineScope = CoroutineScope(Dispatchers.IO),
) : GetTransactions {

    private val transactions: StateFlow<List<TransactionDataAggregate>> =
        transactionStore.walletTransactions(getCurrentWalletId, emptyList())
            .map { items -> items.map { TransactionDataAggregateImpl(it, addressService) } }
            .stateIn(scope, SharingStarted.Eagerly, emptyList())

    override fun transactions(): StateFlow<List<TransactionDataAggregate>> = transactions

    override fun getTransactions(
        filters: List<TransactionsRequestFilter>,
    ): Flow<List<TransactionDataAggregate>> = transactionStore.walletTransactions(getCurrentWalletId, filters)
        .map { items -> items.map { TransactionDataAggregateImpl(it, addressService) } }
        .flowOn(Dispatchers.IO)
}

@Stable
class TransactionDataAggregateImpl(
    data: TransactionExtended,
    addressService: GemAddressService,
) : TransactionDataAggregate {

    private val row = GemTransactionRow(data.toJson())

    override val id: TransactionId = data.transaction.id

    override val asset: Asset = data.asset

    override val title: GemTransactionTitle = row.title()

    override val subtitle: GemTransactionRowSubtitle = row.subtitle()

    override val address: String = subtitle.address()
        ?.let { AddressFormatter(addressService, it, chain = data.transaction.assetId.chain).value() }
        .orEmpty()

    private val coreValue: GemTransactionRowValue = row.value()

    override val valueSign: GemAmountSign = (coreValue as? GemTransactionRowValue.Amount)?.amount?.sign ?: GemAmountSign.NONE

    override val value: String = coreValue.format().orEmpty()

    override val equivalentValue: String? = row.equivalentValue().format()

    override val nftImageUrl: String? = row.nftImageUrl()

    override val type: TransactionType = data.transaction.type

    override val direction: TransactionDirection = data.transaction.direction

    override val pnl: Double? = (coreValue as? GemTransactionRowValue.Pnl)?.value

    override val state: TransactionState = data.transaction.state

    override val createdAt: Long = data.transaction.createdAt
}

private fun GemTransactionRowSubtitle.address(): String? = when (this) {
    is GemTransactionRowSubtitle.ToAddress -> address
    is GemTransactionRowSubtitle.FromAddress -> address
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
