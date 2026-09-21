package com.gemwallet.android.features.confirm.viewmodels

import android.content.Context
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.gemstone.connection.ConnectionStatusObserver
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
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.confirm.models.ConfirmDetailElement
import com.gemwallet.android.features.confirm.viewmodels.localization.broadcastLabel
import com.gemwallet.android.features.confirm.viewmodels.localization.label
import com.gemwallet.android.features.confirm.viewmodels.models.AcquireAssetRequest
import com.gemwallet.android.features.confirm.viewmodels.models.AcquireOptionUIModel
import com.gemwallet.android.features.confirm.viewmodels.models.ConfirmHeaderUIModel
import com.gemwallet.android.features.confirm.viewmodels.models.ConfirmRowUIModel
import com.gemwallet.android.features.confirm.viewmodels.models.FeeSelectionUIModel
import com.gemwallet.android.features.confirm.viewmodels.models.acquireOptions
import com.gemwallet.android.features.confirm.viewmodels.models.buttonState
import com.gemwallet.android.features.confirm.viewmodels.models.confirmHeader
import com.gemwallet.android.features.confirm.viewmodels.models.feeItems
import com.gemwallet.android.features.confirm.viewmodels.models.listItem
import com.gemwallet.android.features.confirm.viewmodels.models.uiModel
import com.gemwallet.android.model.AssetPriceValue
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.FeeAssetSelection
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.actions.FinishConfirmAction
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.models.swap.SwapDetailsUIModelFactory
import com.gemwallet.android.ui.models.swap.SwapDetailsUIModelInput
import com.gemwallet.android.ui.models.swap.SwapProviderUIModelFactory
import com.wallet.core.primitives.ApplicationMetadataSource
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.FeePriority
import com.wallet.core.primitives.Wallet
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapNotNull
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.transformLatest
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
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
import uniffi.gemstone.GemPerpetual
import uniffi.gemstone.GemRefreshKind
import uniffi.gemstone.GemSubmitResult
import uniffi.gemstone.GemTransferAmountResult
import uniffi.gemstone.GemTransferData
import uniffi.gemstone.PerpetualProvider
import uniffi.gemstone.PerpetualType
import uniffi.gemstone.SimulationResult
import uniffi.gemstone.TransactionInputType
import uniffi.gemstone.perpetualConfirmDetails
import uniffi.gemstone.showsFeeAssets
import uniffi.gemstone.swapQuoteSummary
import java.math.BigInteger
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class ConfirmViewModel @Inject constructor(
    private val getSession: GetSession,
    private val confirmService: GemConfirmTransferServiceInterface,
    private val savedStateHandle: SavedStateHandle,
    connectionStatusObserver: ConnectionStatusObserver,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    val refreshIntervalMillis = connectionStatusObserver.refreshIntervalMillis(GemRefreshKind.CONFIRM)
        .stateIn(viewModelScope, SharingStarted.Eagerly, 0L)

    private val reload = MutableSharedFlow<Unit>(extraBufferCapacity = 1)
    val screen = MutableStateFlow(GemConfirmScreen(phase = GemConfirmPhase.LOADING, hasCriticalWarning = false, failure = null))

    val isErrorSheetVisible = MutableStateFlow(false)
    val feeSelection = MutableStateFlow<GemConfirmFeeSelection>(GemConfirmFeeSelection.Priority(FeePriority.Normal.toGem()))
    private val feeAssetSelection = MutableStateFlow<FeeAssetSelection>(FeeAssetSelection.Automatic)
    private var requestSimulation: SimulationResult? = null
    private val requestWallet = MutableStateFlow<Wallet?>(null)

    private val request = savedStateHandle.getStateFlow<String?>(RouteArgument.Params.key, null)
        .filterNotNull()
        .mapNotNull { paramsPack -> unpackTransferData(paramsPack) }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val title = request.map { it?.title() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val isExternalRequest = request.map { it?.inputType?.applicationMetadata != null }
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    val isPaymentRequest = request.map { it?.inputType?.applicationMetadata?.source == ApplicationMetadataSource.Payment }
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    val session = getSession()
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val wallet = combine(requestWallet, session) { requestWallet, session -> requestWallet ?: session?.wallet }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val confirmation = combine(request.filterNotNull(), wallet.filterNotNull()) { request, wallet ->
        confirmService.confirmation(wallet.toGem(), request, requestSimulation).also { screen.value = it.screen() }
    }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val currency = confirmation.filterNotNull()
        .map { it.getCurrency().toPrimitives() }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val networkFeeBuyAmount = confirmation.filterNotNull()
        .map { it.insufficientNetworkFeeBuyAmount() }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, 0)

    private val load = combine(
        confirmation.filterNotNull(),
        feeSelection,
        feeAssetSelection,
        reload.onStart { emit(Unit) },
    ) { session, feeSelection, feeAssetSelection, _ ->
        session to confirmLoadOptions(feeSelection, feeAssetSelection)
    }
        .transformLatest { (session, options) ->
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
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val content = combine(confirmation.filterNotNull(), load.filterNotNull(), currency.filterNotNull()) { session, load, currency ->
        ConfirmContent(session, currency, load)
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val simulation = content
        .map { it?.load?.simulation?.toSimulation(it.session, context) ?: Simulation() }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, Simulation())

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
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val detailElements = combine(request, content, ::buildDetailElements)
        .distinctUntilChanged()
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val transactionRows: StateFlow<List<ConfirmRowUIModel>> = combine(request, content) { request, content ->
        request ?: return@combine emptyList()
        content ?: return@combine emptyList()
        content.session.rowContents(content.load.addressName).mapNotNull { it.uiModel(context) }
    }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val feeUIModel = combine(content, screen) { content, screen ->
        val confirmData = content?.confirmData
        when (val feeRow = screen.feeRow()) {
            GemConfirmFeeRow.Loading -> FeeUIModel.Calculating

            is GemConfirmFeeRow.Unavailable -> FeeUIModel.Unavailable(feeRow.text)

            GemConfirmFeeRow.Ready -> if (content == null || confirmData == null) {
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

    fun init(transfer: GemTransferData, simulationResult: SimulationResult? = null, wallet: Wallet? = null) {
        requestSimulation = simulationResult
        requestWallet.value = wallet
        viewModelScope.launch(ioDispatcher) {
            val pack = transfer.pack()
            if (savedStateHandle.get<String?>(RouteArgument.Params.key) == pack) {
                return@launch
            }
            feeSelection.value = GemConfirmFeeSelection.Priority(transfer.defaultFeePriority())
            screen.update { it.onLoadStarted() }
            savedStateHandle[RouteArgument.Params.key] = pack
        }
    }

    fun dismissErrorSheet() {
        isErrorSheetVisible.value = false
    }

    private fun showError(error: Throwable) {
        screen.update { it.onLoadFailed(error.toConfirmError()) }
        isErrorSheetVisible.value = error.toConfirmError().display().hasInfoSheet()
    }

    val feeListItem: StateFlow<ListItemModel?> = combine(feeUIModel, feeAsset, feeAssets) { fee, asset, assets ->
        fee?.listItem(context, asset?.asset, showsFeeAssetSymbol = showsFeeAssets(assets.map { it.asset.id.toIdentifier() }, asset?.asset?.id?.toIdentifier()))
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val feeItems: StateFlow<List<ListItemModel>> = feeUIModel.map { (it as? FeeUIModel.FeeInfo)?.feeItems(context).orEmpty() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val balanceChangeRows: StateFlow<List<ListItemModel>> = simulation.map { it.balanceChanges.map { change -> change.listItem() } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val acquireOptions: StateFlow<List<AcquireOptionUIModel>> = acquireRequest.map { request -> request?.let { acquireOptions(context, it.buyAmount) }.orEmpty() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val executeErrorText: StateFlow<String?> = screen.map { it.failure?.takeIf { failure -> failure.stage == GemConfirmStage.EXECUTE }?.error?.broadcastLabel(context) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val isLoading: StateFlow<Boolean> = screen.map { it.phase == GemConfirmPhase.LOADING }
        .stateIn(viewModelScope, SharingStarted.Eagerly, true)

    val buttonLabel: StateFlow<String> = button.map { it.kind.label(context) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, context.getString(R.string.transfer_confirm))

    val buttonState: StateFlow<ButtonState> = button.map { it.state.buttonState() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, ButtonState.Loading)

    val header: StateFlow<ConfirmHeaderUIModel?> = combine(amountUIModel, simulation, isPaymentRequest, isLoading, headerAsset) { amount, simulation, isPayment, isLoading, headerAsset ->
        confirmHeader(amount, simulation.header, isPayment, isLoading, headerAsset)
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val feeSelectionUIModel: StateFlow<FeeSelectionUIModel> = feeSelection.map { FeeSelectionUIModel(it.selectedPriority()?.toPrimitives(), it.customGasPrice()) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, FeeSelectionUIModel(FeePriority.Normal, null))

    fun changeFeePriority(priority: FeePriority) = changeFeeSelection(GemConfirmFeeSelection.Priority(priority.toGem()))

    fun changeCustomFee(gasPrice: BigInteger) = changeFeeSelection(GemConfirmFeeSelection.Custom(gasPrice))

    fun feeDetailsModel(currentFee: FeeUIModel.FeeInfo, feeAsset: FeeAssetUIModel): FeeDetailsModel? {
        val confirmData = content.value?.confirmData ?: return null
        return FeeDetailsModel(currentFee, feeAsset, confirmData.feeRateRows(feeSelection.value, feeAsset.asset.toGem()))
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

    fun fetch() {
        screen.update { it.onLoadStarted() }
        reload.tryEmit(Unit)
    }

    fun send(finishAction: FinishConfirmAction) = viewModelScope.launch {
        when (screen.value.action()) {
            GemConfirmAction.LOAD -> {
                fetch()
                return@launch
            }

            GemConfirmAction.EXECUTE -> screen.update { it.onExecuteStarted() }

            null -> return@launch
        }
        val session = confirmation.value ?: return@launch

        try {
            val transactionHash = when (val result = withContext(ioDispatcher) { session.submit() }) {
                is GemSubmitResult.Signed -> result.data.first()
                is GemSubmitResult.Sent -> result.hashes.last()
            }
            finishAction(transactionHash)
        } catch (error: CancellationException) {
            throw error
        } catch (_: GemConfirmException.Cancelled) {
            screen.update { it.onExecuteCancelled() }
        } catch (err: Throwable) {
            screen.update { it.onExecuteFailed(err.toConfirmError()) }
        }
    }

    private data class ConfirmContent(val session: GemConfirmation, val currency: Currency, val load: GemConfirmLoad) {
        val confirmData: GemConfirmData? = load.preload?.confirmData

        val feeAssetUIModel: FeeAssetUIModel =
            FeeAssetUIModel.from(load.feeAsset.toPrimitives(), load.metadata.feeAssetBalance, load.metadata.feePrice(), currency)

        val feeAssets: List<FeeAssetUIModel> = load.feeAssets.map { it.toFeeAssetUIModel(currency) }

        fun assetPrice(asset: Asset): AssetPriceValue = load.metadata.prices.toAssetPriceValue(asset, currency)
    }

    private fun buildDetailElements(request: GemTransferData?, content: ConfirmContent?): List<ConfirmDetailElement> = listOfNotNull(
        buildSwapDetailElement(request, content),
        buildPerpetualDetailElement(request?.inputType?.perpetualType),
    )

    private fun buildPerpetualDetailElement(perpetualType: PerpetualType?): ConfirmDetailElement? = when (val type = perpetualType) {
        null -> null

        is PerpetualType.Modify -> confirmation.value?.autocloseRow(type.data)?.let { ConfirmDetailElement.PerpetualModifyAutoclose(it) }

        else -> perpetualConfirmDetails(type)
            ?.let(ConfirmDetailElement::PerpetualDetails)
    }

    private fun buildSwapDetailElement(transfer: GemTransferData?, content: ConfirmContent?): ConfirmDetailElement.SwapDetails? {
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
