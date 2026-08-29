package com.gemwallet.android.data.coordinators.transaction

import com.gemwallet.android.ext.hash
import uniffi.gemstone.GemExplorerService
import androidx.compose.runtime.Stable
import com.gemwallet.android.application.transactions.cases.GetTransactionDetails
import com.gemwallet.android.application.assets.cases.GetWalletAssets
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.transactions.cases.GetTransaction
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.domains.price.toValueDirection
import com.gemwallet.android.domains.swap.buildAssetRatePair
import com.gemwallet.android.domains.transaction.AmountSign
import com.gemwallet.android.domains.transaction.aggregates.TransactionDetailsAggregate
import com.gemwallet.android.domains.transaction.values.TransactionDetailsValue
import com.gemwallet.android.domains.transaction.values.ValueGroup
import com.gemwallet.android.ext.getAssociatedAssetIds
import com.gemwallet.android.ext.getNftMetadata
import com.gemwallet.android.ext.getPerpetualMetadata
import com.gemwallet.android.ext.getResourceMetadata
import com.gemwallet.android.ext.getSwapMetadata
import com.gemwallet.android.ext.getWalletConnectOutputAction
import com.gemwallet.android.ext.isCompleted
import com.gemwallet.android.math.getRelativeDate
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.CryptoFiatConverter
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.model.TransactionExtended
import com.gemwallet.android.model.ValueFormatter
import com.wallet.core.primitives.AddressType
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.BlockExplorerLink
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.TransactionDirection
import com.wallet.core.primitives.TransactionId
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.TransactionSwapMetadata
import com.wallet.core.primitives.TransactionType
import com.wallet.core.primitives.TransferDataOutputAction
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.mapLatest
import uniffi.gemstone.SwapperProviderMode
import uniffi.gemstone.SwapperProviderType
import uniffi.gemstone.swapperProviderConfig
import uniffi.gemstone.swapperProviderFromStr

@OptIn(ExperimentalCoroutinesApi::class)
class GetTransactionDetailsImpl(
    private val getSession: GetSession,
    private val getTransaction: GetTransaction,
    private val getWalletAssets: GetWalletAssets,
    private val explorerService: GemExplorerService,
) : GetTransactionDetails {

    override fun getTransactionDetails(id: TransactionId): Flow<TransactionDetailsAggregate?> {
        return combine(
            getSession().filterNotNull(),
            getTransaction(id),
        ) { session, data -> Pair(session, data) }
            .flatMapLatest { (session, data) ->
                val ids = data?.transaction?.getAssociatedAssetIds() ?: return@flatMapLatest emptyFlow()
                val swapMetadata = data.transaction.getSwapMetadata()
                val explorerInfo = explorerService.getTransactionLink(
                    chain = data.transaction.assetId.chain.string,
                    hash = data.transaction.hash,
                    provider = swapMetadata?.provider,
                    recipient = data.transaction.to,
                    memo = data.transaction.memo,
                ).let { TransactionDetailsValue.Explorer(it.link, it.name) }
                getWalletAssets(ids).mapLatest { assets ->
                    val swapProvider = swapMetadata?.provider
                        ?.let(::swapperProviderFromStr)
                        ?.let(::swapperProviderConfig)
                    TransactionDetailsAggregateImpl(
                        data = data,
                        associatedAssets = assets,
                        explorer = explorerInfo,
                        currency = session.currency,
                        swapProvider = swapProvider,
                        swapMetadata = swapMetadata,
                        senderExplorerLink = explorerService.getAddressUrl(data.asset.chain.string, data.transaction.from).let { BlockExplorerLink(it.name, it.link) },
                        recipientExplorerLink = explorerService.getAddressUrl(data.asset.chain.string, data.transaction.to).let { BlockExplorerLink(it.name, it.link) },
                    )
                }
            }
            .flowOn(Dispatchers.IO)
    }
}

@Stable
class TransactionDetailsAggregateImpl(
    private val data: TransactionExtended,
    private val associatedAssets: List<AssetInfo>,
    swapMetadata: TransactionSwapMetadata? = null,
    override val explorer: TransactionDetailsValue.Explorer,
    override val currency: Currency,
    private val swapProvider: SwapperProviderType? = null,
    private val senderExplorerLink: BlockExplorerLink? = null,
    private val recipientExplorerLink: BlockExplorerLink? = null,
) : TransactionDetailsAggregate {

    private val swapMetadata = swapMetadata?.takeIf {
        it.fromValue.toBigIntegerOrNull() != null && it.toValue.toBigIntegerOrNull() != null
    }

    override val id: String = data.transaction.id.identifier

    override val asset: Asset = data.asset
    override val type: TransactionType = data.transaction.type
    override val direction: TransactionDirection = data.transaction.direction
    override val state: TransactionState = data.transaction.state

    override val amount: TransactionDetailsValue.Amount
        get() {
            return when (data.transaction.type) {
                TransactionType.Swap -> {
                    val fromAsset = associatedAssets.firstOrNull { it.id() == swapMetadata?.fromAsset }
                    val toAsset = associatedAssets.firstOrNull { it.id() == swapMetadata?.toAsset }

                    if (swapMetadata == null || fromAsset == null || toAsset == null) {
                        TransactionDetailsValue.Amount.None
                    } else {
                        TransactionDetailsValue.Amount.Swap(
                            fromAsset = fromAsset,
                            toAsset = toAsset,
                            fromValue = swapMetadata.fromValue,
                            toValue = swapMetadata.toValue,
                            currency = currency,
                        )
                    }
                }
                TransactionType.TransferNFT -> {
                    data.transaction.getNftMetadata()?.let { TransactionDetailsValue.Amount.NFT(it) }
                        ?: TransactionDetailsValue.Amount.None
                }

                else -> {
                    val value = Crypto(data.transaction.value)
                    val fiat = data.price?.price?.let {
                        CryptoFiatConverter.toFiatString(value, asset.decimals, it, currency)
                    } ?: ""

                    val formatter = ValueFormatter(style = ValueFormatter.Style.Full)

                    val (amount, equivalent) = when (data.transaction.type) {
                        TransactionType.StakeDelegate,
                        TransactionType.StakeUndelegate,
                        TransactionType.StakeRewards,
                        TransactionType.StakeRedelegate,
                        TransactionType.StakeWithdraw,
                        TransactionType.EarnWithdraw,
                        TransactionType.EarnDeposit,
                        TransactionType.Swap,
                        TransactionType.StakeFreeze,
                        TransactionType.StakeUnfreeze -> Pair(formatter.string(value.atomicValue, asset), fiat)
                        TransactionType.Transfer -> Pair(
                            AmountSign(data.transaction.direction).format(formatter.string(value.atomicValue, asset)),
                            fiat,
                        )
                        TransactionType.TransferNFT,
                        TransactionType.AssetActivation,
                        TransactionType.SmartContractCall,
                        TransactionType.PerpetualOpenPosition,
                        TransactionType.PerpetualClosePosition,
                        TransactionType.PerpetualModifyPosition,
                        TransactionType.TokenApproval -> Pair(data.asset.symbol, null)
                    }
                    TransactionDetailsValue.Amount.Plain(data.asset, amount, equivalent)
                }
            }
        }

    override val fee: TransactionDetailsValue.Fee
        get() {
            val fee = Crypto(data.transaction.fee)
            val feeCrypto = ValueFormatter(style = ValueFormatter.Style.Full)
                .string(fee.atomicValue, data.feeAsset)
            val feeFiat = data.feePrice?.price?.let {
                CryptoFiatConverter.toFiatString(fee, data.feeAsset.decimals, it, currency)
            } ?: ""
            return TransactionDetailsValue.Fee(data.feeAsset, feeCrypto, feeFiat)
        }

    override val date: TransactionDetailsValue.Date = TransactionDetailsValue.Date(
        getRelativeDate(data.transaction.createdAt)
    )

    override val status: TransactionDetailsValue.Status = TransactionDetailsValue.Status(data.transaction.state)

    override val estimatedConfirmation: TransactionDetailsValue.EstimatedConfirmation? = data.confirmationEtaSeconds
        ?.takeIf { it > 0u && state == TransactionState.Pending && swapProgress == null }
        ?.let { TransactionDetailsValue.EstimatedConfirmation(it) }

    override val memo: TransactionDetailsValue.Memo? = data.transaction.memo
        ?.takeIf { it.isNotEmpty() }
        ?.let { TransactionDetailsValue.Memo(it) }

    override val resourceType: TransactionDetailsValue.ResourceType? = data.transaction
        .getResourceMetadata()
        ?.resourceType
        ?.let { TransactionDetailsValue.ResourceType(it) }

    override val network: TransactionDetailsValue.Network = TransactionDetailsValue.Network(asset)

    private val perpetualMetadata = data.transaction.getPerpetualMetadata()
    private val usdFormatter = CurrencyFormatter(currency = Currency.USD)

    override val pnl: TransactionDetailsValue.Pnl? = perpetualMetadata?.pnl
        ?.takeIf { it != 0.0 }
        ?.let { TransactionDetailsValue.Pnl(value = "${if (it >= 0) "+" else ""}${usdFormatter.string(it)}", direction = it.toValueDirection()) }

    override val price: TransactionDetailsValue.Price? = perpetualMetadata?.price
        ?.takeIf { it > 0 }
        ?.let { TransactionDetailsValue.Price(usdFormatter.string(it)) }

    override val destination: TransactionDetailsValue.Destination? = when (data.transaction.type) {
        TransactionType.StakeUndelegate,
        TransactionType.StakeRewards,
        TransactionType.StakeRedelegate,
        TransactionType.AssetActivation,
        TransactionType.PerpetualOpenPosition,
        TransactionType.PerpetualClosePosition,
        TransactionType.StakeFreeze,
        TransactionType.StakeUnfreeze,
        TransactionType.PerpetualModifyPosition,
        TransactionType.StakeWithdraw -> null
        TransactionType.TokenApproval -> destinationAddress { address, name, explorerLink ->
            TransactionDetailsValue.Destination.Contract(address, data.asset.chain, name, explorerLink)
        }
        TransactionType.StakeDelegate -> destinationAddress { address, name, explorerLink ->
            TransactionDetailsValue.Destination.Validator(address, data.asset.chain, name, explorerLink)
        }
        TransactionType.SmartContractCall -> destinationAddress { address, name, explorerLink ->
            when (data.transaction.getWalletConnectOutputAction()) {
                TransferDataOutputAction.Send -> TransactionDetailsValue.Destination.Recipient(data = address, chain = data.asset.chain, name = name, explorerLink = explorerLink)
                TransferDataOutputAction.Sign,
                null -> TransactionDetailsValue.Destination.Contract(address, data.asset.chain, name, explorerLink)
            }
        }
        TransactionType.EarnWithdraw,
        TransactionType.EarnDeposit -> destinationAddress { address, name, explorerLink ->
            TransactionDetailsValue.Destination.ProviderAddress(address, data.asset.chain, name, explorerLink)
        }
        TransactionType.Swap -> this@TransactionDetailsAggregateImpl.swapProvider?.name?.let { TransactionDetailsValue.Destination.Provider(it) }
        TransactionType.Transfer,
        TransactionType.TransferNFT -> when (data.transaction.direction) {
            TransactionDirection.SelfTransfer,
            TransactionDirection.Outgoing -> TransactionDetailsValue.Destination.Recipient(
                data = data.transaction.to,
                chain = data.asset.chain,
                name = data.toAddress?.name,
                addressType = data.toAddress?.type,
                explorerLink = recipientExplorerLink,
            )
            TransactionDirection.Incoming -> TransactionDetailsValue.Destination.Sender(
                data = data.transaction.from,
                chain = data.asset.chain,
                name = data.fromAddress?.name,
                addressType = data.fromAddress?.type,
                explorerLink = senderExplorerLink,
            )
        }
    }

    override val valueGroups: List<ValueGroup<TransactionDetailsValue>>
        get() = buildList {
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

    override val swapProgress: TransactionDetailsValue.SwapProgress?
        get() {
            if (type != TransactionType.Swap) return null
            if (state == TransactionState.Confirmed) return null

            val metadata = swapMetadata ?: return null
            val provider = swapProvider?.takeIf { it.mode.isCrossChain } ?: return null

            val fromAsset = data.assets.firstOrNull { it.id == metadata.fromAsset }
                ?: associatedAssets.firstOrNull { it.id() == metadata.fromAsset }?.asset
                ?: data.asset.takeIf { it.id == metadata.fromAsset }
                ?: return null

            return TransactionDetailsValue.SwapProgress(
                fromAsset = fromAsset,
                fromValue = metadata.fromValue,
                providerName = provider.name,
                state = state,
                etaInSeconds = data.confirmationEtaSeconds?.takeIf { it > 0u && !state.isCompleted() },
            )
        }

    override val rate: TransactionDetailsValue.Rate?
        get() {
            val metadata = swapMetadata ?: return null
            val fromAsset = associatedAssets.firstOrNull { it.id() == metadata.fromAsset }?.asset ?: return null
            val toAsset = associatedAssets.firstOrNull { it.id() == metadata.toAsset }?.asset ?: return null
            val rate = buildAssetRatePair(
                fromAsset = fromAsset,
                toAsset = toAsset,
                fromValue = metadata.fromValue,
                toValue = metadata.toValue,
            ) ?: return null
            return TransactionDetailsValue.Rate(rate)
        }

    override val swapAgain: TransactionDetailsValue.SwapAgain?
        get() {
            if (data.transaction.type != TransactionType.Swap) return null
            if (data.transaction.state != TransactionState.Confirmed) return null
            val metadata = swapMetadata ?: return null

            return TransactionDetailsValue.SwapAgain(
                fromAssetId = metadata.fromAsset,
                toAssetId = metadata.toAsset,
            )
        }

    private val participantAddress: String
        get() = when (data.transaction.direction) {
            TransactionDirection.Incoming -> data.transaction.from
            TransactionDirection.Outgoing,
            TransactionDirection.SelfTransfer -> data.transaction.to
        }

    private val participantAddressName: String?
        get() = when (data.transaction.direction) {
            TransactionDirection.Incoming -> data.fromAddress?.name
            TransactionDirection.Outgoing,
            TransactionDirection.SelfTransfer -> data.toAddress?.name
        }

    private val participantExplorerLink: BlockExplorerLink?
        get() = when (participantAddress) {
            data.transaction.from -> senderExplorerLink
            data.transaction.to -> recipientExplorerLink
            else -> null
        }

    private inline fun destinationAddress(
        build: (address: String, name: String?, explorerLink: BlockExplorerLink?) -> TransactionDetailsValue.Destination,
    ): TransactionDetailsValue.Destination? = participantAddress
        .takeIf { it.isNotEmpty() }
        ?.let { build(it, participantAddressName, participantExplorerLink) }
}

private val SwapperProviderMode.isCrossChain: Boolean
    get() = when (this) {
        SwapperProviderMode.OnChain -> false
        SwapperProviderMode.CrossChain,
        SwapperProviderMode.Bridge,
        is SwapperProviderMode.OmniChain -> true
    }
