package com.gemwallet.android.features.confirm.viewmodels

import android.content.Context
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.confirm.cases.BuildConfirmProperties
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.gemstone.di.IoDispatcher
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
import com.gemwallet.android.domains.confirm.paymentInvoice
import com.gemwallet.android.domains.confirm.perpetualType
import com.gemwallet.android.domains.confirm.swapData
import com.gemwallet.android.domains.confirm.toAsset
import com.gemwallet.android.domains.confirm.toFeeAssetUIModel
import com.gemwallet.android.domains.confirm.unpackTransferData
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toAssetPriceValue
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.confirm.models.ConfirmDetailElement
import com.gemwallet.android.features.confirm.models.PerpetualModifyAutocloseFactory
import com.gemwallet.android.features.confirm.viewmodels.localization.broadcastLabel
import com.gemwallet.android.features.confirm.viewmodels.localization.buttonLabel
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
import com.gemwallet.android.features.confirm.viewmodels.models.verificationListItem
import com.gemwallet.android.model.AssetPriceValue
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.FeeAssetSelection
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.actions.FinishConfirmAction
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.models.perpetual.PerpetualConfirmDetailsUIModelFactory
import com.gemwallet.android.ui.models.swap.SwapDetailsUIModelFactory
import com.gemwallet.android.ui.models.swap.SwapDetailsUIModelInput
import com.gemwallet.android.ui.models.swap.SwapProviderUIModelFactory
import com.wallet.core.primitives.AddressName
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.FeePriority
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import java.math.BigInteger
import javax.inject.Inject
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineDispatcher
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
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    private val restart = MutableStateFlow(false)
    val screen = MutableStateFlow(GemConfirmScreen(phase = GemConfirmPhase.LOADING, hasCriticalWarning = false, failure = null))

    val isNetworkFeeSheetVisible = MutableStateFlow(false)
    val isVerificationVisible = MutableStateFlow(false)
    val verificationBridge = PaymentVerificationBridge(::onPaymentVerified)
    val feeSelection = MutableStateFlow<GemConfirmFeeSelection>(GemConfirmFeeSelection.Priority(FeePriority.Normal.toGem()))
    private val feeAssetSelection = MutableStateFlow<FeeAssetSelection>(FeeAssetSelection.Automatic)
    private val assetSelection = MutableStateFlow<AssetId?>(null)
    private var requestSimulation: SimulationResult? = null

    private val request = savedStateHandle.getStateFlow<String?>(RouteArgument.Params.key, null)
        .filterNotNull()
        .mapNotNull { paramsPack -> unpackTransferData(paramsPack) }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val session = getSession()
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val confirmation = combine(request.filterNotNull(), session.filterNotNull()) { request, session ->
        confirmService.confirmation(session.wallet.toGem(), request, requestSimulation).also { screen.value = it.screen() }
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
        assetSelection,
        restart,
    ) { session, feeSelection, feeAssetSelection, assetSelection, _ ->
        session to confirmLoadOptions(feeSelection, feeAssetSelection, assetSelection)
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
    .flowOn(ioDispatcher)
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val content = combine(confirmation.filterNotNull(), load.filterNotNull(), currency.filterNotNull()) { session, load, currency ->
        ConfirmContent(session, currency, load)
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val transfer = combine(request, load) { request, load -> load?.transfer ?: request }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val title = transfer.map { it?.title() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val isExternalRequest = transfer.map { it?.inputType?.applicationMetadata != null || it?.inputType is TransactionInputType.Payment }
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    val verification = transfer.map { it?.verification() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val isPaymentPlaceholder = transfer.map { it?.inputType is TransactionInputType.Payment && it.value == BigInteger.ZERO }
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    val paymentAssetIds = transfer.map { it?.inputType?.paymentInvoice?.quotes?.mapNotNull { quote -> quote.assetId.toAssetId() }.orEmpty() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val simulation = content
        .map { it?.load?.simulation?.toSimulation(it.session, context) ?: Simulation() }
        .flowOn(ioDispatcher)
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

    private val assetPrice = combine(transfer, content) { transfer, content -> transfer?.asset?.let { content?.assetPrice(it) } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val transferAmount = content.map { content ->
        when (val amount = content?.load?.preload?.amount ?: return@map null) {
            is GemTransferAmountResult.Amount -> amount.amount.value
            is GemTransferAmountResult.Error -> null
        }
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val amountUIModel = combine(transfer, content, transferAmount) { transfer, content, transferAmount ->
        val inputType = transfer?.inputType ?: return@combine null
        content ?: return@combine null

        val amount = Crypto(transferAmount ?: transfer.value)

        AmountUIModel(
            headerKind = transfer.headerKind(),
            amount = amount.atomicValue,
            fromAsset = content.assetPrice(transfer.asset),
            fromAmount = amount.atomicValue,
            toAsset = inputType.toAsset?.let(content::assetPrice),
            toAmount = inputType.swapData?.quote?.toValue,
            nftAsset = inputType.nftAsset,
            currency = content.currency,
            paymentPrice = inputType.paymentInvoice?.price,
        )
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val headerAsset = combine(transfer, amountUIModel) { transfer, amount ->
        when (transfer?.inputType) {
            is TransactionInputType.Withdrawal -> GemPerpetual(PerpetualProvider.HYPERCORE).use { it.depositAsset() }.toPrimitives()
            else -> amount?.asset
        }
    }
    .flowOn(ioDispatcher)
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val detailElements = combine(transfer, content, ::buildDetailElements)
        .distinctUntilChanged()
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val transactionProperties = combine(transfer, session, content) { transfer, session, content ->
        transfer ?: return@combine emptyList()
        session ?: return@combine emptyList()
        buildConfirmProperties(transfer, session.wallet, content?.addressName)
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
        assetSelection.value = null
        viewModelScope.launch(ioDispatcher) {
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

    fun showVerification() {
        isVerificationVisible.value = true
    }

    fun dismissVerification() {
        isVerificationVisible.value = false
    }

    private fun onPaymentVerified() {
        isVerificationVisible.value = false
        reload()
    }

    private fun showError(error: Throwable) {
        screen.update { it.onLoadFailed(error.toConfirmError()) }
        isNetworkFeeSheetVisible.value = error is GemConfirmException.InsufficientNetworkFee
    }

    val feeListItem: StateFlow<ListItemModel?> = combine(feeUIModel, feeAsset, verification) { fee, asset, verification ->
        verification?.let { verificationListItem(context) } ?: fee?.listItem(context, asset?.asset)
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

    val buttonLabel: StateFlow<String> = combine(screen, button) { screen, button -> screen.buttonLabel(context, button.kind) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, context.getString(R.string.transfer_confirm))

    val buttonState: StateFlow<ButtonState> = button.map { it.state.buttonState() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, ButtonState.Loading)

    val header: StateFlow<ConfirmHeaderUIModel?> = combine(amountUIModel, simulation, isPaymentPlaceholder, isLoading, headerAsset) { amount, simulation, isPlaceholder, isLoading, headerAsset ->
        confirmHeader(amount, simulation.header, isPlaceholder, isLoading, headerAsset)
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

    fun changePaymentAsset(assetId: AssetId) {
        if (transfer.value?.asset?.id == assetId) return
        screen.update { it.onLoadStarted() }
        assetSelection.update { assetId }
    }

    fun reload() = restart.update { !it }

    fun changeFeeAsset(assetId: AssetId) {
        if (feeAsset.value?.asset?.id == assetId) return
        val selection = FeeAssetSelection.Selected(assetId)
        if (selection == feeAssetSelection.value) return
        screen.update { it.onLoadStarted() }
        feeAssetSelection.update { selection }
    }

    fun send(finishAction: FinishConfirmAction) = viewModelScope.launch(ioDispatcher) {
        when (screen.value.action()) {
            GemConfirmAction.LOAD -> {
                reload()
                return@launch
            }
            GemConfirmAction.EXECUTE -> screen.update { it.onExecuteStarted() }
            null -> return@launch
        }
        val session = confirmation.value ?: return@launch

        try {
            val (transactionHash, warning) = when (val result = session.execute()) {
                is GemExecuteResult.Signed -> result.data.first() to result.warning
                is GemExecuteResult.Sent -> result.hashes.last() to result.warning
            }
            viewModelScope.launch(Dispatchers.Main) {
                finishAction(transactionHash, warning?.text(context))
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
