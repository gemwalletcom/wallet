package com.gemwallet.android.data.coordinators.transaction

import androidx.compose.runtime.Stable
import com.gemwallet.android.application.transactions.cases.GetTransactions
import com.gemwallet.android.application.transactions.cases.TransactionsRequestFilter
import com.gemwallet.android.application.session.cases.GetCurrentWalletId
import com.gemwallet.android.data.services.gemstone.stores.GemstoneTransactionStore
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.domains.transaction.AmountSign
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
import uniffi.gemstone.GemTransactionFormatter
import uniffi.gemstone.GemTransactionSubtitle
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
    scope: CoroutineScope = CoroutineScope(Dispatchers.IO),
) : GetTransactions {

    private val transactions: StateFlow<List<TransactionDataAggregate>> =
        transactionStore.walletTransactions(getCurrentWalletId, emptyList())
            .map { items -> items.map { TransactionDataAggregateImpl(it) } }
            .stateIn(scope, SharingStarted.Eagerly, emptyList())

    override fun transactions(): StateFlow<List<TransactionDataAggregate>> = transactions

    override fun getTransactions(
        filters: List<TransactionsRequestFilter>,
    ): Flow<List<TransactionDataAggregate>> = transactionStore.walletTransactions(getCurrentWalletId, filters)
        .map { items -> items.map { TransactionDataAggregateImpl(it) } }
        .flowOn(Dispatchers.IO)
}

@Stable
class TransactionDataAggregateImpl(
    private val data: TransactionExtended,
) : TransactionDataAggregate {

    override val id: TransactionId = data.transaction.id

    override val asset: Asset = data.asset

    override val subtitle: GemTransactionSubtitle = transactionFormatter.subtitle(data.transaction.toJson())

    override val addressName: String? = subtitle.address()?.let { address ->
        listOfNotNull(data.fromAddress, data.toAddress).firstOrNull { it.address == address }?.name
    }

    override val address: String = subtitle.address()
        ?.let { AddressFormatter(it, chain = data.transaction.assetId.chain).value() }
        .orEmpty()

    override val value: String get() = when (data.transaction.type) {
        TransactionType.Swap -> {
            getSwapValue(true)?.let { (value, asset) ->
                AmountSign.Incoming.format(formatter.string(value, asset))
            } ?: ""
        }
        TransactionType.PerpetualOpenPosition -> usdFiatFormatter.string(
            CryptoFiatConverter.toFiat(Crypto(data.transaction.value), HypercoreUSDC.decimals, price = 1.0).atomicValue,
        )
        TransactionType.PerpetualClosePosition -> pnl?.let {
            PriceChangeFormatter(usdFiatFormatter).string(it)
        } ?: ""
        TransactionType.StakeUndelegate,
        TransactionType.StakeRewards,
        TransactionType.StakeRedelegate,
        TransactionType.StakeWithdraw,
        TransactionType.EarnWithdraw,
        TransactionType.StakeDelegate,
        TransactionType.EarnDeposit,
        TransactionType.StakeFreeze,
        TransactionType.StakeUnfreeze -> getFormattedValue()
        TransactionType.Transfer -> AmountSign(data.transaction.direction).format(getFormattedValue())
        TransactionType.TokenApproval -> data.asset.symbol
        TransactionType.TransferNFT,
        TransactionType.AssetActivation,
        TransactionType.SmartContractCall,
        TransactionType.PerpetualModifyPosition
            -> ""
    }

    override val equivalentValue: String? get() = when (data.transaction.type) {
        TransactionType.Swap -> getSwapValue(false)?.let { (value, asset) ->
            AmountSign.Outgoing.format(formatter.string(value, asset))
        }
        else -> null
    }

    override val nftImageUrl: String? = data.transaction.getNftMetadata()?.getImageUrl()

    override val title: GemTransactionTitle = transactionFormatter.title(data.transaction.toJson())

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

private val transactionFormatter = GemTransactionFormatter()

fun GemTransactionSubtitle.address(): String? = when (this) {
    is GemTransactionSubtitle.ToAddress -> address
    is GemTransactionSubtitle.FromAddress -> address
    is GemTransactionSubtitle.ToResource,
    is GemTransactionSubtitle.FromResource,
    is GemTransactionSubtitle.Price,
    GemTransactionSubtitle.None -> null
}
