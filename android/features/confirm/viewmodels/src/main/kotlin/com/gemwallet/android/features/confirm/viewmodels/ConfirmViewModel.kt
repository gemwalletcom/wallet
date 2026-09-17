package com.gemwallet.android.features.confirm.viewmodels

import android.content.Context
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.confirm.cases.BuildConfirmProperties
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.domains.confirm.AmountUIModel
import com.gemwallet.android.domains.confirm.FeeAssetUIModel
import com.gemwallet.android.domains.confirm.FeeDetailsModel
import com.gemwallet.android.domains.confirm.FeeUIModel
import com.gemwallet.android.domains.confirm.applicationMetadata
import com.gemwallet.android.domains.confirm.asset
import com.gemwallet.android.domains.confirm.confirmLoadOptions
import com.gemwallet.android.domains.confirm.nftAsset
import com.gemwallet.android.domains.confirm.pack
import com.gemwallet.android.domains.confirm.perpetualType
import com.gemwallet.android.domains.confirm.swapData
import com.gemwallet.android.domains.confirm.toAsset
import com.gemwallet.android.domains.confirm.toFeeAssetUIModel
import com.gemwallet.android.domains.confirm.unpackTransferData
import com.gemwallet.android.ext.toAssetPriceValue
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.confirm.models.ConfirmDetailElement
import com.gemwallet.android.features.confirm.models.PerpetualModifyAutocloseFactory
import com.gemwallet.android.features.confirm.viewmodels.models.AcquireAssetRequest
import com.gemwallet.android.features.confirm.viewmodels.models.AcquireOptionUIModel
import com.gemwallet.android.features.confirm.viewmodels.models.ConfirmRowUIModel
import com.gemwallet.android.features.confirm.viewmodels.models.acquireOptions
import com.gemwallet.android.features.confirm.viewmodels.models.feeItems
import com.gemwallet.android.features.confirm.viewmodels.models.listItem
import com.gemwallet.android.features.confirm.viewmodels.models.uiModel
import com.gemwallet.android.model.AssetPriceValue
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.FeeAssetSelection
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.models.actions.FinishConfirmAction
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.models.perpetual.PerpetualConfirmDetailsUIModelFactory
import com.gemwallet.android.ui.models.swap.SwapDetailsUIModelFactory
import com.gemwallet.android.ui.models.swap.SwapDetailsUIModelInput
import com.gemwallet.android.ui.models.swap.SwapProviderUIModelFactory
import com.wallet.core.primitives.AddressName
import com.wallet.core.primitives.ApplicationMetadataSource
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.FeePriority
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import javax.inject.Inject
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapNotNull
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.gemstone.GemAcquireAssetFlow
import uniffi.gemstone.GemConfirmAction
import uniffi.gemstone.GemConfirmButton
import uniffi.gemstone.GemConfirmButtonKind
import uniffi.gemstone.GemConfirmButtonState
import uniffi.gemstone.GemConfirmData
import uniffi.gemstone.GemConfirmException
import uniffi.gemstone.GemConfirmFeeRow
import uniffi.gemstone.GemConfirmFeeSelection
import uniffi.gemstone.GemConfirmLoad
import uniffi.gemstone.GemConfirmPhase
import uniffi.gemstone.GemConfirmScreen
import uniffi.gemstone.GemConfirmStage
import uniffi.gemstone.GemConfirmTransferServiceInterface
import uniffi.gemstone.GemConfirmation
import uniffi.gemstone.GemExecuteResult
import uniffi.gemstone.GemPerpetual
import uniffi.gemstone.GemTransferAmountResult
import uniffi.gemstone.GemTransferData
import uniffi.gemstone.PerpetualProvider
import uniffi.gemstone.PerpetualType
import uniffi.gemstone.SimulationResult
import uniffi.gemstone.TransactionInputType
import uniffi.gemstone.perpetualDetails
import uniffi.gemstone.swapQuoteSummary

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class ConfirmViewModel @Inject constructor(
    private val getSession: GetSession,
    private val buildConfirmProperties: BuildConfirmProperties,
    private val confirmService: GemConfirmTransferServiceInterface,
    private val savedStateHandle: SavedStateHandle,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    private val restart = MutableStateFlow(false)
    val screen = MutableStateFlow(GemConfirmScreen(phase = GemConfirmPhase.LOADING, hasCriticalWarning = false, failure = null))

    val isNetworkFeeSheetVisible = MutableStateFlow(false)
    val feeSelection = MutableStateFlow<GemConfirmFeeSelection>(GemConfirmFeeSelection.Priority(FeePriority.Normal.toGem()))
    private val feeAssetSelection = MutableStateFlow<FeeAssetSelection>(FeeAssetSelection.Automatic)
    private var requestSimulation: SimulationResult? = null

    private val request = savedStateHandle.getStateFlow<String?>(RouteArgument.Params.key, null)
        .filterNotNull()
        .mapNotNull { paramsPack -> unpackTransferData(paramsPack) }
        .flowOn(Dispatchers.Default)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val title = request.map { it?.title() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val isExternalRequest = request.map { it?.inputType?.applicationMetadata != null }
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    val isPaymentRequest = request.map { it?.inputType?.applicationMetadata?.source == ApplicationMetadataSource.Payment }
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    val session = getSession()
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val confirmation = combine(request.filterNotNull(), session.filterNotNull()) { request, session ->
        confirmService.confirmation(session.wallet.toGem(), request, requestSimulation).also { screen.value = it.screen() }
    }
    .flowOn(Dispatchers.IO)
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val currency = confirmation.filterNotNull()
        .map { it.getCurrency().toPrimitives() }
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val networkFeeBuyAmount = confirmation.filterNotNull()
        .map { it.insufficientNetworkFeeBuyAmount() }
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, 0)

    private val load = combine(
        confirmation.filterNotNull(),
        feeSelection,
        feeAssetSelection,
        restart,
    ) { session, feeSelection, feeAssetSelection, _ ->
        session to confirmLoadOptions(feeSelection, feeAssetSelection)
    }
    .flatMapLatest { (session, options) ->
        flow {
            screen.update { it.onLoadStarted() }
            try {
                emit(session.state())
                val load = session.load(options)
                emit(load)
                screen.update { it.onLoaded(load) }
            } catch (error: CancellationException) {
                throw error
            } catch (err: Throwable) {
                showError(err)
            }
        }
    }
    .flowOn(Dispatchers.IO)
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val content = combine(confirmation.filterNotNull(), load.filterNotNull(), currency.filterNotNull()) { session, load, currency ->
        ConfirmContent(session, currency, load)
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val simulation = content
        .map { it?.load?.simulation?.toSimulation(it.session, context) ?: Simulation() }
        .flowOn(Dispatchers.Default)
        .stateIn(viewModelScope, SharingStarted.Eagerly, Simulation())

    val payloadAddressNames = content
        .map { content ->
            content?.load?.simulation?.addressNames.orEmpty()
                .map { it.toPrimitives() }
                .filter { it.name.isNotEmpty() && !it.name.equals(it.address, ignoreCase = true) }
                .associate { it.address.lowercase() to it.name }
        }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyMap())

    val button = screen.map { it.button() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, GemConfirmButton(GemConfirmButtonKind.CONFIRM, GemConfirmButtonState.LOADING))

    val feeAsset = content.map { it?.feeAssetUIModel }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val feeAssets = content.map { it?.feeAssets.orEmpty() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    private val assetPrice = combine(request, content) { request, content -> request?.asset?.let { content?.assetPrice(it) } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val transferAmount = content.map { content ->
        when (val amount = content?.load?.preload?.amount ?: return@map null) {
            is GemTransferAmountResult.Amount -> amount.amount.value
            is GemTransferAmountResult.Error -> null
        }
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val amountUIModel = combine(request, content, transferAmount) { request, content, transferAmount ->
        val inputType = request?.inputType ?: return@combine null
        content ?: return@combine null

        val amount = Crypto(transferAmount ?: request.value)

        AmountUIModel(
            headerKind = request.headerKind(),
            amount = amount.atomicValue,
            fromAsset = content.assetPrice(request.asset),
            fromAmount = amount.atomicValue,
            toAsset = inputType.toAsset?.let(content::assetPrice),
            toAmount = inputType.swapData?.quote?.toValue,
            nftAsset = inputType.nftAsset,
            currency = content.currency,
        )
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val headerAsset = combine(request, amountUIModel) { request, amount ->
        when (request?.inputType) {
            is TransactionInputType.Withdrawal -> GemPerpetual(PerpetualProvider.HYPERCORE).use { it.depositAsset() }.toPrimitives()
            else -> amount?.asset
        }
    }
    .flowOn(Dispatchers.IO)
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val detailElements = combine(request, content, ::buildDetailElements)
        .distinctUntilChanged()
        .flowOn(Dispatchers.Default)
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val transactionProperties = combine(request, session, content) { request, session, content ->
        request ?: return@combine emptyList()
        session ?: return@combine emptyList()
        buildConfirmProperties(request, session.wallet, content?.addressName)
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val transactionRows: StateFlow<List<ConfirmRowUIModel>> = transactionProperties.map { properties -> properties.map { it.uiModel(context) } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val feeUIModel = combine(content, screen) { content, screen ->
        val confirmData = content?.confirmData
        when (screen.feeRow()) {
            GemConfirmFeeRow.LOADING -> FeeUIModel.Calculating
            GemConfirmFeeRow.UNAVAILABLE -> FeeUIModel.Error
            GemConfirmFeeRow.READY -> if (content == null || confirmData == null) {
                FeeUIModel.Calculating
            } else {
                FeeUIModel.FeeInfo(
                    amount = confirmData.fee.fee,
                    additionalFees = confirmData.additionalFees,
                    feeAsset = content.feeAssetUIModel.asset,
                    price = content.feeAssetUIModel.price?.price?.price,
                    currency = content.currency,
                    priority = confirmData.selectedPriority.toPrimitives(),
                )
            }
        }
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val feeValue = feeUIModel.map { (it as? FeeUIModel.FeeInfo)?.cryptoAmountWithFiat.orEmpty() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, "")

    private val acquireRequestState = MutableStateFlow<AcquireAssetRequest?>(null)
    val acquireRequest = acquireRequestState.asStateFlow()

    val loadError = combine(screen, feeUIModel, assetPrice, networkFeeBuyAmount) { screen, fee, assetPrice, buyAmount ->
        screen.failure?.takeIf { it.stage == GemConfirmStage.LOAD }?.error?.display()?.uiModel(
            context = context,
            fee = fee as? FeeUIModel.FeeInfo,
            assetPrice = assetPrice,
            networkFeeBuyAmount = buyAmount,
            acquireFlow = ::acquireFlow,
            onAcquire = ::acquire,
        )
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    fun acquire(asset: Asset, buyAmount: Int?) {
        acquireRequestState.value = AcquireAssetRequest(asset, buyAmount, offersOptions = acquireFlow(asset) == GemAcquireAssetFlow.OPTIONS)
    }

    fun dismissAcquire() = acquireRequestState.update { null }


    fun init(transfer: GemTransferData, simulationResult: SimulationResult? = null) {
        requestSimulation = simulationResult
        feeSelection.value = GemConfirmFeeSelection.Priority(transfer.defaultFeePriority())
        viewModelScope.launch(Dispatchers.IO) {
            val pack = transfer.pack()
            if (savedStateHandle.get<String?>(RouteArgument.Params.key) == pack) {
                return@launch
            }
            screen.update { it.onLoadStarted() }
            savedStateHandle[RouteArgument.Params.key] = pack
        }
    }

    fun dismissNetworkFeeSheet() {
        isNetworkFeeSheetVisible.value = false
    }

    private fun showError(error: Throwable) {
        screen.update { it.onLoadFailed(error.toConfirmError()) }
        isNetworkFeeSheetVisible.value = error is GemConfirmException.InsufficientNetworkFee
    }

    val feeListItem: StateFlow<ListItemModel?> = combine(feeUIModel, feeAsset) { fee, asset -> fee?.listItem(context, asset?.asset) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val feeItems: StateFlow<List<ListItemModel>> = feeUIModel.map { (it as? FeeUIModel.FeeInfo)?.feeItems(context).orEmpty() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val balanceChangeRows: StateFlow<List<ListItemModel>> = simulation.map { it.balanceChanges.map { change -> change.listItem() } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val acquireOptions: StateFlow<List<AcquireOptionUIModel>> = acquireRequest.map { request -> request?.let { acquireOptions(context, it.buyAmount) }.orEmpty() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    fun feeDetailsModel(currentFee: FeeUIModel.FeeInfo, feeAsset: FeeAssetUIModel, selection: GemConfirmFeeSelection): FeeDetailsModel? {
        val confirmData = content.value?.confirmData ?: return null
        return FeeDetailsModel(currentFee, feeAsset, confirmData.feeRateRows(selection, feeAsset.asset.toGem()))
    }

    fun changeFeeSelection(selection: GemConfirmFeeSelection) {
        if (selection == feeSelection.value) return
        screen.update { it.onLoadStarted() }
        feeSelection.update { selection }
    }

    fun changeFeeAsset(assetId: AssetId) {
        if (feeAsset.value?.asset?.id == assetId) return
        val selection = FeeAssetSelection.Selected(assetId)
        if (selection == feeAssetSelection.value) return
        screen.update { it.onLoadStarted() }
        feeAssetSelection.update { selection }
    }

    fun send(finishAction: FinishConfirmAction) = viewModelScope.launch(Dispatchers.IO) {
        when (screen.value.action()) {
            GemConfirmAction.LOAD -> {
                restart.update { !it }
                return@launch
            }
            GemConfirmAction.EXECUTE -> screen.update { it.onExecuteStarted() }
            null -> return@launch
        }
        val session = confirmation.value ?: return@launch

        try {
            val transactionHash = when (val result = session.execute()) {
                is GemExecuteResult.Signed -> result.data.first()
                is GemExecuteResult.Sent -> result.hashes.last()
            }
            viewModelScope.launch(Dispatchers.Main) {
                finishAction(transactionHash)
            }
        } catch (error: CancellationException) {
            throw error
        } catch (_: GemConfirmException.Cancelled) {
            screen.update { it.onExecuteCancelled() }
        } catch (err: Throwable) {
            screen.update { it.onExecuteFailed(err.toConfirmError()) }
        }
    }

    private data class ConfirmContent(
        val session: GemConfirmation,
        val currency: Currency,
        val load: GemConfirmLoad,
    ) {
        val confirmData: GemConfirmData? = load.preload?.confirmData

        val feeAssetUIModel: FeeAssetUIModel =
            FeeAssetUIModel.from(load.feeAsset.toPrimitives(), load.metadata.feeAssetBalance, load.metadata.feePrice(), currency)

        val feeAssets: List<FeeAssetUIModel> = load.feeAssets.map { it.toFeeAssetUIModel(currency) }

        val addressName: AddressName? = load.addressName?.toPrimitives()

        fun assetPrice(asset: Asset): AssetPriceValue = load.metadata.prices.toAssetPriceValue(asset, currency)
    }

    private fun buildDetailElements(
        request: GemTransferData?,
        content: ConfirmContent?,
    ): List<ConfirmDetailElement> {
        return listOfNotNull(
            buildSwapDetailElement(request, content),
            buildPerpetualDetailElement(request?.inputType?.perpetualType),
        )
    }

    private fun buildPerpetualDetailElement(
        perpetualType: PerpetualType?,
    ): ConfirmDetailElement? = when (val type = perpetualType) {
        null -> null
        is PerpetualType.Modify -> confirmation.value?.let { PerpetualModifyAutocloseFactory.create(type.data, it) }
        else -> perpetualDetails(type)
            ?.let(PerpetualConfirmDetailsUIModelFactory::create)
            ?.let(ConfirmDetailElement::PerpetualDetails)
    }

    private fun buildSwapDetailElement(
        transfer: GemTransferData?,
        content: ConfirmContent?,
    ): ConfirmDetailElement.SwapDetails? {
        val swapData = transfer?.inputType?.swapData ?: return null
        content ?: return null
        val fromAsset = content.assetPrice(transfer.asset)
        val toAsset = transfer.inputType.toAsset?.let(content::assetPrice) ?: return null
        val summary = swapQuoteSummary(swapData.quote, fromAsset.asset.toGem(), toAsset.asset.toGem())

        val provider = SwapProviderUIModelFactory.create(
            providerId = swapData.quote.providerData.provider,
            title = swapData.quote.providerData.protocolName,
            receiveAsset = toAsset,
            toValue = swapData.quote.toValue,
        )
        val model = SwapDetailsUIModelFactory.create(
            SwapDetailsUIModelInput(
                payAsset = fromAsset,
                receiveAsset = toAsset,
                summary = summary,
                provider = provider,
                slippageBps = swapData.quote.slippageBps,
                selectedSlippage = swapData.quote.slippageBps,
                isProviderSelectable = false,
                priceImpact = fromAsset.swapValue(transfer.value)
                    .priceImpact(toAsset.swapValue(swapData.quote.toValue)),
            ),
        ) ?: return null

        return ConfirmDetailElement.SwapDetails(model)
    }

    private fun acquireFlow(asset: Asset): GemAcquireAssetFlow = requireNotNull(confirmation.value).acquireAssetFlow(asset.chain.string)
}

private fun Throwable.toConfirmError(): GemConfirmException = this as? GemConfirmException ?: GemConfirmException.Load(msg = message.orEmpty())
