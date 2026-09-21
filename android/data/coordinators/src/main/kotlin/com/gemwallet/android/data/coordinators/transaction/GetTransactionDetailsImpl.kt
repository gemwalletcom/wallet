package com.gemwallet.android.data.coordinators.transaction

import androidx.compose.runtime.Stable
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.transactions.cases.GetTransaction
import com.gemwallet.android.application.transactions.cases.GetTransactionDetails
import com.gemwallet.android.domains.swap.AssetRateFormatter
import com.gemwallet.android.domains.transaction.aggregates.TransactionDetailsAggregate
import com.gemwallet.android.domains.transaction.values.TransactionDetailsValue
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.AssetPriceInfo
import com.gemwallet.android.model.AssetPriceValue
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.CryptoFiatConverter
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.model.ValueFormatter
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.NFTAssetId
import com.wallet.core.primitives.TransactionDirection
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
import uniffi.gemstone.BlockExplorerLink
import uniffi.gemstone.GemTransactionAmount
import uniffi.gemstone.GemTransactionDetailRow
import uniffi.gemstone.GemTransactionDetailRows
import uniffi.gemstone.GemTransactionDetailSection
import uniffi.gemstone.GemTransactionDetailsService
import uniffi.gemstone.GemTransactionDetailsServiceInterface
import uniffi.gemstone.GemTransactionHeader
import uniffi.gemstone.GemTransactionHeaderAction
import uniffi.gemstone.GemTransactionParticipant
import uniffi.gemstone.GemTransactionParticipantRole
import uniffi.gemstone.GemTransactionTitle
import uniffi.gemstone.GemValueStyle
import uniffi.gemstone.transactionDetailSections

class GetTransactionDetailsImpl(private val getSession: GetSession, private val getTransaction: GetTransaction, private val transactionDetailsService: GemTransactionDetailsServiceInterface) : GetTransactionDetails {

    override fun getTransactionDetails(id: TransactionId): Flow<TransactionDetailsAggregate?> = combine(
        getSession().filterNotNull(),
        getTransaction(id),
    ) { session, data -> Pair(session, data) }
        .mapNotNull { (session, data) ->
            data?.let {
                TransactionDetailsAggregateImpl(
                    transactionDetailsService.detailRows(it.toGem(), session.wallet.type.toGem()),
                    session.currency,
                )
            }
        }
        .flowOn(Dispatchers.IO)
}

@Stable
class TransactionDetailsAggregateImpl(private val rows: GemTransactionDetailRows, override val currency: Currency) : TransactionDetailsAggregate {

    private val valueFormatter = ValueFormatter(style = GemValueStyle.AUTO)
    private val rateFormatter = AssetRateFormatter()

    override val id: String = rows.id

    override val asset: Asset = rows.asset.toPrimitives()
    override val title: GemTransactionTitle = rows.title

    override val type: TransactionType = rows.transactionType.toPrimitives()
    override val direction: TransactionDirection = rows.direction.toPrimitives()
    override val state: TransactionState = rows.state.toPrimitives()

    val amount: TransactionDetailsValue.Amount = when (val header = rows.header) {
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

    override val fee: TransactionDetailsValue.Fee = TransactionDetailsValue.Fee(rows.feeRow)

    val estimatedConfirmation: TransactionDetailsValue.EstimatedConfirmation? = rows.estimatedConfirmationSeconds
        ?.let { TransactionDetailsValue.EstimatedConfirmation(it) }

    val participant: TransactionDetailsValue.Destination? = rows.participant?.destination()

    override val explorer: BlockExplorerLink = rows.explorer

    val swapProgress: TransactionDetailsValue.SwapProgress? = rows.swapProgress?.let(TransactionDetailsValue::SwapProgress)

    val rate: TransactionDetailsValue.Rate? = rows.rate?.let { TransactionDetailsValue.Rate(rateFormatter.format(it)) }

    val swapAgain: TransactionDetailsValue.SwapAgain? = rows.swapAgain
        ?.let { TransactionDetailsValue.SwapAgain(fromAssetId = AssetId(it.fromAssetId), toAssetId = AssetId(it.toAssetId)) }

    override val sections: List<GemTransactionDetailSection> = transactionDetailSections(rows)

    override fun value(row: GemTransactionDetailRow): TransactionDetailsValue = when (row) {
        GemTransactionDetailRow.Header -> amount
        GemTransactionDetailRow.SwapProgress -> requireNotNull(swapProgress)
        GemTransactionDetailRow.SwapAgain -> requireNotNull(swapAgain)
        GemTransactionDetailRow.EstimatedConfirmation -> requireNotNull(estimatedConfirmation)
        GemTransactionDetailRow.Participant -> requireNotNull(participant)
        GemTransactionDetailRow.Rate -> requireNotNull(rate)
        GemTransactionDetailRow.Fee -> fee
        is GemTransactionDetailRow.Row -> TransactionDetailsValue.Row(row.row)
    }

    private fun GemTransactionAmount.plain(showsFiat: Boolean): TransactionDetailsValue.Amount.Plain {
        val asset = asset.toPrimitives()
        return TransactionDetailsValue.Amount.Plain(
            asset = asset,
            value = sign.format(valueFormatter.string(value, asset)),
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
        val chain = asset.id.chain
        val name = name?.toPrimitives()
        val link = link.toPrimitives()
        return when (role) {
            GemTransactionParticipantRole.SENDER -> TransactionDetailsValue.Destination.Sender(address, text, chain, name?.type, link)
            GemTransactionParticipantRole.RECIPIENT -> TransactionDetailsValue.Destination.Recipient(address, text, chain, name?.type, link)
            GemTransactionParticipantRole.CONTRACT -> TransactionDetailsValue.Destination.Contract(address, text, chain, link)
            GemTransactionParticipantRole.VALIDATOR -> TransactionDetailsValue.Destination.Validator(address, text, chain, link)
            GemTransactionParticipantRole.PROVIDER -> TransactionDetailsValue.Destination.ProviderAddress(address, text, chain, link)
        }
    }
}
