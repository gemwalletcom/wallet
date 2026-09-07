package com.gemwallet.android.data.coordinators.transaction

import androidx.compose.runtime.Stable
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.transactions.cases.GetTransaction
import com.gemwallet.android.application.transactions.cases.GetTransactionDetails
import com.gemwallet.android.domains.price.toValueDirection
import com.gemwallet.android.domains.swap.buildAssetRatePair
import com.gemwallet.android.domains.transaction.aggregates.TransactionDetailsAggregate
import com.gemwallet.android.domains.transaction.format
import com.gemwallet.android.domains.transaction.values.TransactionDetailsValue
import com.gemwallet.android.domains.transaction.values.ValueGroup
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.math.getRelativeDate
import com.gemwallet.android.model.AssetPriceInfo
import com.gemwallet.android.model.AssetPriceValue
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.CryptoFiatConverter
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.model.ValueFormatter
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.NFTAssetId
import com.wallet.core.primitives.TransactionDirection
import com.wallet.core.primitives.TransactionExtended
import com.wallet.core.primitives.TransactionId
import com.wallet.core.primitives.TransactionNFTTransferMetadata
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.TransactionType
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.mapNotNull
import uniffi.gemstone.GemTransactionAmount
import uniffi.gemstone.GemTransactionDetailRows
import uniffi.gemstone.GemTransactionDetailsService
import uniffi.gemstone.GemTransactionHeader
import uniffi.gemstone.GemTransactionHeaderAction
import uniffi.gemstone.GemTransactionParticipant
import uniffi.gemstone.GemTransactionParticipantRole
import uniffi.gemstone.GemTransactionTitle

class GetTransactionDetailsImpl(
    private val getSession: GetSession,
    private val getTransaction: GetTransaction,
    private val transactionDetailsService: GemTransactionDetailsService,
) : GetTransactionDetails {

    override fun getTransactionDetails(id: TransactionId): Flow<TransactionDetailsAggregate?> = combine(
        getSession().filterNotNull(),
        getTransaction(id),
    ) { session, data -> Pair(session, data) }
        .mapNotNull { (session, data) ->
            data?.let { TransactionDetailsAggregateImpl(it, transactionDetailsService.detailRows(it.toJson()), session.currency) }
        }
        .flowOn(Dispatchers.IO)
}

@Stable
class TransactionDetailsAggregateImpl(
    private val data: TransactionExtended,
    private val rows: GemTransactionDetailRows,
    override val currency: Currency,
) : TransactionDetailsAggregate {

    private val fullFormatter = ValueFormatter(style = ValueFormatter.Style.Full)
    private val usdFormatter = CurrencyFormatter(currency = Currency.USD)

    override val id: String = data.transaction.id.identifier

    override val asset: Asset = data.asset
    override val title: GemTransactionTitle = rows.title

    override val type: TransactionType = data.transaction.type
    override val direction: TransactionDirection = data.transaction.direction
    override val state: TransactionState = data.transaction.state

    override val amount: TransactionDetailsValue.Amount = when (val header = rows.header) {
        is GemTransactionHeader.Amount -> header.amount.plain(showsFiat = header.showsFiat)
        is GemTransactionHeader.Swap -> TransactionDetailsValue.Amount.Swap(
            fromAsset = header.from.priceValue(),
            fromValue = header.from.value,
            toAsset = header.to.priceValue(),
            toValue = header.to.value,
            currency = currency,
        )
        is GemTransactionHeader.Nft -> TransactionDetailsValue.Amount.NFT(
            TransactionNFTTransferMetadata(assetId = NFTAssetId(header.assetId), name = header.name),
        )
        is GemTransactionHeader.Symbol -> header.asset.toPrimitives().let { TransactionDetailsValue.Amount.Plain(it, it.symbol, null) }
        is GemTransactionHeader.AssetImage -> header.asset.toPrimitives().let { TransactionDetailsValue.Amount.Plain(it, it.symbol, null) }
    }

    override val headerAction: GemTransactionHeaderAction? = rows.headerAction

    override val fee: TransactionDetailsValue.Fee = rows.fee.let { fee ->
        val asset = fee.asset.toPrimitives()
        TransactionDetailsValue.Fee(asset, fullFormatter.string(fee.value, asset), fee.fiat(asset).orEmpty())
    }

    override val date: TransactionDetailsValue.Date = TransactionDetailsValue.Date(getRelativeDate(data.transaction.createdAt))

    override val status: TransactionDetailsValue.Status = TransactionDetailsValue.Status(data.transaction.state)

    override val estimatedConfirmation: TransactionDetailsValue.EstimatedConfirmation? = rows.estimatedConfirmationSeconds
        ?.let { TransactionDetailsValue.EstimatedConfirmation(it) }

    override val memo: TransactionDetailsValue.Memo? = rows.memo?.let { TransactionDetailsValue.Memo(it) }

    override val resourceType: TransactionDetailsValue.ResourceType? = rows.resource
        ?.let { TransactionDetailsValue.ResourceType(it.toPrimitives()) }

    override val network: TransactionDetailsValue.Network = TransactionDetailsValue.Network(asset)

    override val pnl: TransactionDetailsValue.Pnl? = rows.pnl
        ?.let { TransactionDetailsValue.Pnl(value = "${if (it >= 0) "+" else ""}${usdFormatter.string(it)}", direction = it.toValueDirection()) }

    override val price: TransactionDetailsValue.Price? = rows.price?.let { TransactionDetailsValue.Price(usdFormatter.string(it)) }

    override val destination: TransactionDetailsValue.Destination? = rows.providerName?.let { TransactionDetailsValue.Destination.Provider(it) }
        ?: rows.participant?.destination()

    override val explorer: TransactionDetailsValue.Explorer = TransactionDetailsValue.Explorer(rows.explorer.link, rows.explorer.name)

    override val swapProgress: TransactionDetailsValue.SwapProgress? = rows.swapProgress?.let { progress ->
        TransactionDetailsValue.SwapProgress(
            fromAsset = progress.fromAsset.toPrimitives(),
            fromValue = progress.fromValue,
            providerName = progress.providerName,
            transfer = progress.transfer,
            swap = progress.swap,
            etaInSeconds = progress.etaSeconds,
        )
    }

    override val rate: TransactionDetailsValue.Rate? = rows.rate?.let { rate ->
        buildAssetRatePair(
            fromAsset = rate.from.asset.toPrimitives(),
            toAsset = rate.to.asset.toPrimitives(),
            fromValue = rate.from.value,
            toValue = rate.to.value,
        )?.let { TransactionDetailsValue.Rate(it) }
    }

    override val swapAgain: TransactionDetailsValue.SwapAgain? = rows.swapAgain
        ?.let { TransactionDetailsValue.SwapAgain(fromAssetId = AssetId(it.fromAssetId), toAssetId = AssetId(it.toAssetId)) }

    override val valueGroups: List<ValueGroup<TransactionDetailsValue>> = buildList {
        add(ValueGroup(listOf(amount)))
        swapProgress?.let { add(ValueGroup(listOf(it))) }
        swapAgain?.let { add(ValueGroup(listOf(it))) }
        val providerDestination = destination as? TransactionDetailsValue.Destination.Provider
        val addressDestination = if (providerDestination == null) destination else null
        add(
            ValueGroup(
                listOfNotNull(
                    date,
                    status,
                    estimatedConfirmation,
                    rate,
                    addressDestination,
                    resourceType,
                    network,
                    providerDestination,
                    pnl,
                    price,
                )
            )
        )
        add(ValueGroup(listOf(fee)))
        add(ValueGroup(listOf(explorer)))
    }

    private fun GemTransactionAmount.plain(showsFiat: Boolean): TransactionDetailsValue.Amount.Plain {
        val asset = asset.toPrimitives()
        return TransactionDetailsValue.Amount.Plain(
            asset = asset,
            value = sign.format(fullFormatter.string(value, asset)),
            equivalent = fiat(asset).takeIf { showsFiat }.orEmpty(),
        )
    }

    private fun GemTransactionAmount.fiat(asset: Asset): String? = price?.let {
        CryptoFiatConverter.toFiatString(Crypto(value), asset.decimals, it.price, currency)
    }

    private fun GemTransactionAmount.priceValue(): AssetPriceValue = AssetPriceValue(
        asset = asset.toPrimitives(),
        price = price?.let { AssetPriceInfo(currency, it.toPrimitives()) },
    )

    private fun GemTransactionParticipant.destination(): TransactionDetailsValue.Destination {
        val chain = data.asset.id.chain
        val name = name?.toPrimitives()
        val link = link.toPrimitives()
        return when (role) {
            GemTransactionParticipantRole.SENDER -> TransactionDetailsValue.Destination.Sender(address, chain, name?.name, name?.type, link)
            GemTransactionParticipantRole.RECIPIENT -> TransactionDetailsValue.Destination.Recipient(address, chain, name?.name, name?.type, link)
            GemTransactionParticipantRole.CONTRACT -> TransactionDetailsValue.Destination.Contract(address, chain, name?.name, link)
            GemTransactionParticipantRole.VALIDATOR -> TransactionDetailsValue.Destination.Validator(address, chain, name?.name, link)
            GemTransactionParticipantRole.PROVIDER -> TransactionDetailsValue.Destination.ProviderAddress(address, chain, name?.name, link)
        }
    }
}
