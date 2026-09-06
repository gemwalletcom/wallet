package com.gemwallet.android.features.confirm.viewmodels

import com.gemwallet.android.ui.R
import uniffi.gemstone.GemTransferAmountResult
import uniffi.gemstone.GemConfirmException
import uniffi.gemstone.GemTransferData
import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.domains.confirm.asset
import com.gemwallet.android.domains.confirm.nftAsset
import com.gemwallet.android.domains.confirm.pack
import com.gemwallet.android.domains.confirm.perpetualType
import com.gemwallet.android.domains.confirm.swapData
import com.gemwallet.android.domains.confirm.toAsset
import com.gemwallet.android.domains.confirm.confirmLoadOptions
import com.gemwallet.android.domains.confirm.toGem
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.confirm.cases.BuildConfirmProperties
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.ext.toAssetPriceValue
import com.gemwallet.android.ext.toCurrency
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.AssetPriceValue
import uniffi.gemstone.GemConfirmButton
import uniffi.gemstone.GemConfirmData
import uniffi.gemstone.GemConfirmButtonKind
import uniffi.gemstone.GemConfirmButtonState
import uniffi.gemstone.GemConfirmFeeRow
import uniffi.gemstone.GemConfirmLoad
import uniffi.gemstone.GemConfirmPhase
import uniffi.gemstone.GemConfirmScreen
import uniffi.gemstone.GemAcquireAssetFlow
import uniffi.gemstone.GemConfirmTransferServiceInterface
import uniffi.gemstone.GemExecuteResult
import uniffi.gemstone.GemSwapQuoteSummary
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.FeeSelection
import com.gemwallet.android.model.FeeAssetSelection
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.models.perpetual.PerpetualConfirmDetailsUIModelFactory
import com.gemwallet.android.ui.models.swap.SwapDetailsUIModelFactory
import com.gemwallet.android.ui.models.swap.SwapDetailsUIModelInput
import com.gemwallet.android.ui.models.swap.SwapProviderUIModelFactory
import com.gemwallet.android.ui.models.actions.FinishConfirmAction
import com.gemwallet.android.domains.confirm.AmountUIModel
import com.gemwallet.android.domains.confirm.FeeAssetUIModel
import com.gemwallet.android.domains.confirm.toFeeAssetUIModel
import com.gemwallet.android.features.confirm.models.ConfirmDetailElement
import com.gemwallet.android.features.confirm.models.PerpetualModifyAutocloseFactory
import com.gemwallet.android.domains.confirm.ConfirmState
import com.gemwallet.android.domains.confirm.FeeDetailsModel
import com.gemwallet.android.domains.confirm.FeeUIModel
import com.wallet.core.primitives.AddressName
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PerpetualType
import com.wallet.core.primitives.FeePriority
import com.wallet.core.primitives.TransactionType
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapNotNull
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import com.wallet.core.primitives.SimulationResult
import java.math.BigInteger
import javax.inject.Inject
import com.gemwallet.android.domains.confirm.unpackTransferData

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class ConfirmViewModel @Inject constructor(
    private val getSession: GetSession,
    private val buildConfirmProperties: BuildConfirmProperties,
    private val confirmService: GemConfirmTransferServiceInterface,
    private val savedStateHandle: SavedStateHandle,
) : ViewModel() {

    private val restart = MutableStateFlow(false)
    val state = MutableStateFlow<ConfirmState>(ConfirmState.Prepare)

    val isNetworkFeeSheetVisible = MutableStateFlow(false)
    val feeSelection = MutableStateFlow<FeeSelection>(FeeSelection.Preset(FeePriority.Normal))
    private val feeAssetSelection = MutableStateFlow<FeeAssetSelection>(FeeAssetSelection.Automatic)
    private var requestSimulation: String? = null

    private val request = savedStateHandle.getStateFlow<String?>(RouteArgument.Params.key, null)
        .filterNotNull()
        .mapNotNull { paramsPack -> unpackTransferData(paramsPack) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val session = getSession()
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val currency = flow { emit(confirmService.getCurrency().toCurrency()) }
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val confirmSession = combine(request.filterNotNull(), session.filterNotNull()) { request, session ->
        confirmService.session(session.wallet.toGem(), request, requestSimulation)
    }
    .flowOn(Dispatchers.IO)
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val load = combine(
        confirmSession.filterNotNull(),
        feeSelection,
        feeAssetSelection,
        restart,
    ) { session, feeSelection, feeAssetSelection, _ ->
        session to confirmLoadOptions(feeSelection, feeAssetSelection)
    }
    .flatMapLatest { (session, options) ->
        flow {
            state.update { ConfirmState.Prepare }
            try {
                emit(session.state())
                emit(session.load(options))
                state.update { ConfirmState.Ready }
            } catch (error: CancellationException) {
                throw error
            } catch (_: GemConfirmException.AccountMissing) {
                state.update { ConfirmState.FatalError(R.string.errors_wallet_account_missing) }
            } catch (err: Throwable) {
                showError(err)
            }
        }
    }
    .flowOn(Dispatchers.IO)
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val content = combine(load.filterNotNull(), currency.filterNotNull()) { load, currency ->
        ConfirmContent(currency, load)
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val simulation = content
        .map { it?.load?.simulation?.toSimulation(confirmService) ?: Simulation() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, Simulation())

    val payloadAddressNames = content
        .map { content ->
            content?.load?.simulation?.addressNames.orEmpty()
                .map { it.toPrimitives() }
                .filter { it.name.isNotEmpty() && !it.name.equals(it.address, ignoreCase = true) }
                .associate { it.address.lowercase() to it.name }
        }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyMap())

    private val screen = combine(state, content, simulation) { state, content, simulation ->
        GemConfirmScreen(
            phase = state.phase(content),
            amountFailed = content?.load?.preload?.amount is GemTransferAmountResult.Error,
            hasCriticalWarning = simulation.hasCriticalWarning,
        )
    }

    val button = screen.map { it.button() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, GemConfirmButton(GemConfirmButtonKind.CONFIRM, GemConfirmButtonState.LOADING))

    val feeAsset = content.map { it?.feeAssetUIModel }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val feeAssets = content.map { it?.feeAssets.orEmpty() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val assetPrice = combine(request, content) { request, content -> request?.asset?.let { content?.assetPrice(it) } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val transferAmount = content.map { content ->
        when (val amount = content?.load?.preload?.amount ?: return@map null) {
            is GemTransferAmountResult.Amount -> amount.amount.value
            is GemTransferAmountResult.Error -> {
                showError(amount.error)
                null
            }
        }
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val amountUIModel = combine(request, content, transferAmount) { request, content, transferAmount ->
        val inputType = request?.inputType ?: return@combine null
        content ?: return@combine null

        val amount = Crypto(transferAmount ?: request.value)

        AmountUIModel(
            transactionType = request.transactionType().toPrimitives(),
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

    val perpetualType = request
        .map { it?.inputType?.perpetualType }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val detailElements = combine(request, content, ::buildDetailElements)
        .distinctUntilChanged()
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val transactionProperties = combine(request, session, content) { request, session, content ->
        request ?: return@combine emptyList()
        session ?: return@combine emptyList()
        buildConfirmProperties(request, session.wallet, content?.addressName)
    }
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


    fun init(transfer: GemTransferData, simulationResult: SimulationResult? = null) {
        requestSimulation = simulationResult?.toJson()
        feeSelection.value = FeeSelection.Preset(transfer.defaultFeePriority().toPrimitives())
        viewModelScope.launch(Dispatchers.IO) {
            val pack = transfer.pack()
            if (savedStateHandle.get<String?>(RouteArgument.Params.key) == pack) {
                return@launch
            }
            state.update { ConfirmState.Prepare }
            savedStateHandle[RouteArgument.Params.key] = pack
        }
    }

    fun dismissNetworkFeeSheet() {
        isNetworkFeeSheetVisible.value = false
    }

    private fun showError(error: Throwable) {
        state.update { ConfirmState.Error(error) }
        isNetworkFeeSheetVisible.value = error is GemConfirmException.InsufficientNetworkFee
    }

    fun feeDetailsModel(currentFee: FeeUIModel.FeeInfo, feeAsset: FeeAssetUIModel, selection: FeeSelection): FeeDetailsModel? {
        val confirmData = content.value?.confirmData ?: return null
        return FeeDetailsModel(currentFee, feeAsset, confirmData.feeRateRows(selection.toGem(), feeAsset.asset.toGem()))
    }

    fun changeFeeSelection(selection: FeeSelection) {
        if (selection == feeSelection.value) return
        state.update { ConfirmState.Prepare }
        feeSelection.update { selection }
    }

    fun changeFeeAsset(assetId: AssetId) {
        if (feeAsset.value?.asset?.id == assetId) return
        val selection = FeeAssetSelection.Selected(assetId)
        if (selection == feeAssetSelection.value) return
        state.update { ConfirmState.Prepare }
        feeAssetSelection.update { selection }
    }

    fun send(finishAction: FinishConfirmAction) = viewModelScope.launch(Dispatchers.IO) {
        if (state.value is ConfirmState.Error) {
            restart.update { !it }
            return@launch
        }
        state.update { ConfirmState.Sending }

        val preload = content.value?.load?.preload

        try {
            if (preload == null) {
                error("confirm input is not loaded")
            }
            val amount = when (val calculated = preload.amount) {
                is GemTransferAmountResult.Amount -> calculated.amount.value
                is GemTransferAmountResult.Error -> throw calculated.error
            }
            val transactionHash = execute(preload.confirmData, amount)
            state.update { ConfirmState.Result(transactionHash = transactionHash) }
            viewModelScope.launch(Dispatchers.Main) {
                finishAction(transactionHash)
            }
        } catch (error: CancellationException) {
            throw error
        } catch (_: GemConfirmException.Cancelled) {
            state.update { ConfirmState.Ready }
        } catch (err: Throwable) {
            state.update { ConfirmState.BroadcastError(err) }
        }
    }

    private suspend fun execute(confirmData: GemConfirmData, value: BigInteger): String {
        val result = confirmService.execute(
            confirm = confirmData,
            value = value,
            networkFee = confirmData.fee.fee,
            simulation = requestSimulation,
        )
        return when (result) {
            is GemExecuteResult.Signed -> result.data.first()
            is GemExecuteResult.Sent -> result.hashes.last()
        }
    }

    private data class ConfirmContent(
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
        is PerpetualType.Modify -> PerpetualModifyAutocloseFactory.create(type.content, confirmService)
        else -> PerpetualConfirmDetailsUIModelFactory.create(type)?.let(ConfirmDetailElement::PerpetualDetails)
    }

    private fun buildSwapDetailElement(
        transfer: GemTransferData?,
        content: ConfirmContent?,
    ): ConfirmDetailElement.SwapDetails? {
        val swapData = transfer?.inputType?.swapData ?: return null
        content ?: return null
        val fromAsset = content.assetPrice(transfer.asset)
        val toAsset = transfer.inputType.toAsset?.let(content::assetPrice) ?: return null
        val summary = GemSwapQuoteSummary(swapData.quote)

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
                fromValue = transfer.value,
                toValue = swapData.quote.toValue,
                provider = provider,
                slippageBps = swapData.quote.slippageBps,
                selectedSlippage = swapData.quote.slippageBps,
                etaInSeconds = swapData.quote.etaInSeconds,
                isProviderSelectable = false,
                priceImpact = fromAsset.swapValue(transfer.value)
                    .priceImpact(toAsset.swapValue(swapData.quote.toValue))
                    ?.toPrimitives(),
                minReceiveValue = summary.minReceiveValue(),
                etaMinutes = summary.etaMinutes(),
            ),
        ) ?: return null

        return ConfirmDetailElement.SwapDetails(model)
    }

    fun acquireFlow(asset: Asset): GemAcquireAssetFlow = confirmService.acquireAssetFlow(asset.chain.string)

    private fun ConfirmState.phase(content: ConfirmContent?): GemConfirmPhase = when (this) {
        ConfirmState.Prepare -> GemConfirmPhase.LOADING
        ConfirmState.Ready -> GemConfirmPhase.READY
        ConfirmState.Sending, is ConfirmState.Result -> GemConfirmPhase.CONFIRMING
        is ConfirmState.Error -> if (content?.load?.preload?.amount is GemTransferAmountResult.Error) GemConfirmPhase.READY else GemConfirmPhase.FAILED
        is ConfirmState.BroadcastError, is ConfirmState.FatalError -> GemConfirmPhase.FAILED
    }
}
