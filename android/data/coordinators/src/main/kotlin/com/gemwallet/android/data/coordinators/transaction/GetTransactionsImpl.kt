package com.gemwallet.android.data.coordinators.transaction

import androidx.compose.runtime.Stable
import com.gemwallet.android.application.transactions.cases.GetTransactions
import com.gemwallet.android.application.transactions.cases.TransactionsRequestFilter
import com.gemwallet.android.application.session.cases.GetCurrentWalletId
import com.gemwallet.android.data.services.gemstone.stores.GemstoneTransactionStore
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.domains.transaction.aggregates.TransactionDataAggregate
import com.gemwallet.android.domains.asset.getImageUrl
import com.gemwallet.android.ext.AddressFormatter
import com.gemwallet.android.ext.HypercoreUSDC
import com.gemwallet.android.ext.getNftMetadata
import com.gemwallet.android.ext.getPerpetualMetadata
import com.gemwallet.android.ext.getSwapMetadata
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.CryptoFiatConverter
import com.gemwallet.android.model.TransactionExtended
import com.gemwallet.android.model.ValueFormatter
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.model.PriceChangeFormatter
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.TransactionDirection
import com.wallet.core.primitives.TransactionId
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.TransactionType
import com.gemwallet.android.serializer.toJson
import uniffi.gemstone.GemAddressService
import uniffi.gemstone.GemTransactionSummary
import com.gemwallet.android.domains.transaction.format
import com.gemwallet.android.domains.transaction.sign
import uniffi.gemstone.GemAmountSign
import uniffi.gemstone.GemTransactionSubtitle
import uniffi.gemstone.GemTransactionValue
import uniffi.gemstone.GemTransactionTitle
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import java.math.BigInteger

private val usdFiatFormatter = CurrencyFormatter(type = CurrencyFormatter.Type.Fiat, currency = Currency.USD)

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
    private val data: TransactionExtended,
    private val addressService: GemAddressService,
) : TransactionDataAggregate {

    private val row = GemTransactionSummary(data.transaction.toJson())

    override val id: TransactionId = data.transaction.id

    override val asset: Asset = data.asset

    override val subtitle: GemTransactionSubtitle = row.subtitle()

    override val addressName: String? = subtitle.address()?.let { address ->
        listOfNotNull(data.fromAddress, data.toAddress).firstOrNull { it.address == address }?.name
    }

    override val address: String = subtitle.address()
        ?.let { AddressFormatter(addressService, it, chain = data.transaction.assetId.chain).value() }
        .orEmpty()

    private val coreValue: GemTransactionValue = row.value()

    override val valueSign: GemAmountSign = coreValue.sign()

    override val value: String = amount(coreValue).orEmpty()

    override val equivalentValue: String? = amount(row.equivalentValue())

    private fun amount(value: GemTransactionValue): String? = when (value) {
        GemTransactionValue.None -> null
        GemTransactionValue.AssetSymbol -> data.asset.symbol
        is GemTransactionValue.Amount -> value.sign.format(getFormattedValue())
        GemTransactionValue.SwapReceived -> swapAmount(toAsset = true, sign = GemAmountSign.INCOMING)
        GemTransactionValue.SwapSpent -> swapAmount(toAsset = false, sign = GemAmountSign.OUTGOING)
        GemTransactionValue.PerpetualNotional -> usdFiatFormatter.string(
            CryptoFiatConverter.toFiat(Crypto(data.transaction.value), HypercoreUSDC.decimals, price = 1.0).atomicValue,
        )
        is GemTransactionValue.PerpetualPnl -> PriceChangeFormatter(usdFiatFormatter).string(value.value)
    }

    private fun swapAmount(toAsset: Boolean, sign: GemAmountSign): String? =
        getSwapValue(toAsset)?.let { (value, asset) -> sign.format(formatter.string(value, asset)) }

    override val nftImageUrl: String? = data.transaction.getNftMetadata()?.getImageUrl()

    override val title: GemTransactionTitle = row.title()

    override val type: TransactionType = data.transaction.type

    override val direction: TransactionDirection  = data.transaction.direction

    private val perpetualMetadata = data.transaction.getPerpetualMetadata()

    override val pnl: Double? = perpetualMetadata?.pnl

    override val state: TransactionState = data.transaction.state
    override val createdAt: Long = data.transaction.createdAt

    private fun getSwapValue(toAsset: Boolean): Pair<BigInteger, Asset>? {
        val swapMetadata = data.transaction.getSwapMetadata() ?: return null
        val (value, assetId) = if (toAsset) {
            Pair(swapMetadata.toValue, swapMetadata.toAsset)
        } else {
            Pair(swapMetadata.fromValue, swapMetadata.fromAsset)
        }
        val asset = data.assets.firstOrNull { assetId == it.id } ?: return null

        return value.toBigIntegerOrNull()?.let { Pair(it, asset) }
    }

    private fun getFormattedValue(): String =
        formatter.string(data.transaction.value.toBigInteger(), data.asset)

    private companion object {
        val formatter = ValueFormatter(style = ValueFormatter.Style.Short)
    }
}

fun GemTransactionSubtitle.address(): String? = when (this) {
    is GemTransactionSubtitle.ToAddress -> address
    is GemTransactionSubtitle.FromAddress -> address
    is GemTransactionSubtitle.ToResource,
    is GemTransactionSubtitle.FromResource,
    is GemTransactionSubtitle.Price,
    GemTransactionSubtitle.None -> null
}
