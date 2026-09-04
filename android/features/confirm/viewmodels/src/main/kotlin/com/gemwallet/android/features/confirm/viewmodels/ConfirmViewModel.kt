package com.gemwallet.android.features.confirm.viewmodels

import com.gemwallet.android.ui.R
import uniffi.gemstone.GemTransferAmountResult
import uniffi.gemstone.GemConfirmException
import uniffi.gemstone.GemTransferData
import uniffi.gemstone.GemTransferServiceInterface
import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.domains.confirm.asset
import com.gemwallet.android.domains.confirm.nftAsset
import com.gemwallet.android.domains.confirm.pack
import com.gemwallet.android.domains.confirm.perpetualType
import com.gemwallet.android.domains.confirm.swapData
import com.gemwallet.android.domains.confirm.toAsset
import com.gemwallet.android.domains.confirm.unpack
import com.gemwallet.android.domains.swap.providerId
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.confirm.cases.BuildConfirmProperties
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.blockchain.services.confirmLoadOptions
import com.gemwallet.android.blockchain.services.toGem
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.blockchain.services.toSignerParams
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.ext.toAssetPriceValue
import com.gemwallet.android.ext.toCurrency
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.AssetPriceValue
import uniffi.gemstone.GemConfirmMetadata
import uniffi.gemstone.GemConfirmSimulationState
import uniffi.gemstone.GemAcquireAssetFlow
import uniffi.gemstone.GemConfirmTransferServiceInterface
import uniffi.gemstone.GemExecuteResult
import uniffi.gemstone.GemSwapQuoteSummary
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.FeeSelection
import com.gemwallet.android.model.FeeAssetSelection
import com.gemwallet.android.model.SignerParams
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.models.perpetual.PerpetualConfirmDetailsUIModelFactory
import com.gemwallet.android.ui.models.swap.SwapDetailsUIModelFactory
import com.gemwallet.android.ui.models.swap.SwapDetailsUIModelInput
import com.gemwallet.android.ui.models.swap.SwapProviderUIModelFactory
import com.gemwallet.android.ui.models.actions.FinishConfirmAction
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.buttonState
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

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class ConfirmViewModel @Inject constructor(
    private val getSession: GetSession,
    private val buildConfirmProperties: BuildConfirmProperties,
    private val confirmService: GemConfirmTransferServiceInterface,
    private val savedStateHandle: SavedStateHandle,
    private val transferService: GemTransferServiceInterface,
) : ViewModel() {

    private val restart = MutableStateFlow(false)
    val state = MutableStateFlow<ConfirmState>(ConfirmState.Prepare)
    val feeSelection = MutableStateFlow<FeeSelection>(FeeSelection.Preset(FeePriority.Normal))
    private val feeAssetSelection = MutableStateFlow<FeeAssetSelection>(FeeAssetSelection.Automatic)
    private val simulationResult = MutableStateFlow<SimulationResult?>(null)

    private val request = savedStateHandle.getStateFlow<String?>(RouteArgument.Params.key, null)
        .filterNotNull()
        .mapNotNull { paramsPack -> transferService.unpack(paramsPack) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val session = getSession()
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val currency = flow { emit(confirmService.getCurrency().toCurrency()) }
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val initialContent = combine(request.filterNotNull(), currency.filterNotNull()) { transfer, currency ->
        val initial = confirmService.initialState(transfer.inputType, simulationResult.value?.toJson())
        ConfirmContent(
            currency = currency,
            metadata = initial.metadata,
            feeAsset = initial.feeAsset.toPrimitives(),
            simulation = GemConfirmSimulationState(simulation = initial.simulation, addressNames = emptyList()),
        )
    }
    .flowOn(Dispatchers.IO)
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val preloadData = combine(
        request.filterNotNull(),
        currency.filterNotNull(),
        feeSelection,
        feeAssetSelection,
        restart,
    ) { request, currency, feeSelection, feeAssetSelection, _ ->
        state.update { ConfirmState.Prepare }
        val input = try {
            confirmService.confirmInput(request)
        } catch (_: GemConfirmException.AccountMissing) {
            state.update { ConfirmState.FatalError(R.string.errors_wallet_account_missing) }
            return@combine null
        }

        val preload = try {
            val load = confirmService.load(
                input = input,
                options = confirmLoadOptions(feeSelection, feeAssetSelection),
                simulation = simulationResult.value?.toJson(),
            )
            load.preload.confirmData.simulation?.let { simulationResult.value = it.decodeJson() }
            Preload(
                signerParams = load.preload.toSignerParams(),
                amount = load.preload.amount,
                content = ConfirmContent(
                    currency = currency,
                    metadata = load.preload.metadata,
                    feeAsset = load.preload.feeAsset.toPrimitives(),
                    simulation = load.simulation,
                ),
                feeAssets = load.feeAssets.map { it.toFeeAssetUIModel(currency) },
            )
        } catch (error: CancellationException) {
            throw error
        } catch (err: Throwable) {
            state.update { ConfirmState.Error(err) }
            return@combine null
        }

        state.update { ConfirmState.Ready }

        preload
    }
    .flowOn(Dispatchers.IO)
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val content = combine(initialContent, preloadData) { initial, preload -> preload?.content ?: initial }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val simulation = combine(content, simulationResult, request) { content, result, params ->
        content?.simulation?.toSimulation(
            warnings = result?.warnings.orEmpty(),
            chain = params?.inputType?.asset?.id?.chain,
            confirmService = confirmService,
        ) ?: Simulation()
    }.stateIn(viewModelScope, SharingStarted.Eagerly, Simulation())

    val payloadAddressNames = content
        .map { content ->
            content?.simulation?.addressNames.orEmpty()
                .map { it.decodeJson<AddressName>() }
                .filter { it.name.isNotEmpty() && !it.name.equals(it.address, ignoreCase = true) }
                .associate { it.address.lowercase() to it.name }
        }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyMap())

    val buttonState = combine(state, simulation) { state, simulation ->
        buttonState(
            enabled = state !is ConfirmState.Prepare
                && state !is ConfirmState.Sending
                && !simulation.hasCriticalWarning,
            loading = state is ConfirmState.Sending || state is ConfirmState.Prepare || state is ConfirmState.Result,
        )
    }.stateIn(viewModelScope, SharingStarted.Eagerly, ButtonState.Loading)

    val feeAsset = content.map { it?.feeAssetUIModel }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val feeAssets = preloadData.map { it?.feeAssets.orEmpty() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    private val transferAmount = preloadData.map { preload ->
        if (preload == null) return@map null
        when (val amount = preload.amount) {
            is GemTransferAmountResult.Amount -> amount.amount.value
            is GemTransferAmountResult.Error -> {
                state.update { ConfirmState.Error(amount.error) }
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
            transactionType = inputType.transactionType().decodeJson<TransactionType>(),
            headerKind = inputType.headerKind(),
            amount = amount.atomicValue,
            fromAsset = content.assetPrice(inputType.asset),
            fromAmount = amount.atomicValue,
            toAsset = inputType.toAsset?.let(content::assetPrice),
            toAmount = inputType.swapData?.quote?.toValue?.let(::BigInteger),
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

    private val recipientAddressName = request
        .filterNotNull()
        .map { it.inputType.asset.id.chain to it.recipient.address.takeIf { address -> address.isNotEmpty() } }
        .distinctUntilChanged()
        .map { (chain, address) ->
            address?.let { confirmService.addressName(chain.string, it)?.decodeJson<AddressName>() }
        }
        .flowOn(Dispatchers.IO)

    val transactionProperties = combine(request, session, recipientAddressName) { request, session, addressName ->
        request ?: return@combine emptyList()
        session ?: return@combine emptyList()
        buildConfirmProperties(request, session.wallet, addressName)
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val feeUIModel = combine(preloadData, content, state) { preload, content, state ->
        val signerParams = preload?.signerParams
        val amount = signerParams?.fee?.amount
        val feeAsset = content?.feeAssetUIModel
        if (state is ConfirmState.Prepare) {
            FeeUIModel.Calculating
        } else if (amount == null || feeAsset == null) {
            if (state is ConfirmState.Error) FeeUIModel.Error else FeeUIModel.Calculating
        } else {
            FeeUIModel.FeeInfo(
                amount = amount,
                feeAsset = feeAsset.asset,
                price = feeAsset.price?.price?.price,
                currency = content.currency,
                priority = signerParams.fee.priority,
            )
        }
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val feeValue = feeUIModel.map { (it as? FeeUIModel.FeeInfo)?.cryptoAmountWithFiat.orEmpty() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, "")


    fun init(transfer: GemTransferData, simulationResult: SimulationResult? = null) {
        this.simulationResult.value = simulationResult
        feeSelection.value = FeeSelection.Preset(transfer.inputType.defaultFeePriority().toPrimitives())
        viewModelScope.launch(Dispatchers.IO) {
            val pack = transferService.pack(transfer)
            if (savedStateHandle.get<String?>(RouteArgument.Params.key) == pack) {
                return@launch
            }
            state.update { ConfirmState.Prepare }
            savedStateHandle[RouteArgument.Params.key] = pack
        }
    }

    fun feeDetailsModel(currentFee: FeeUIModel.FeeInfo, feeAsset: FeeAssetUIModel, selection: FeeSelection): FeeDetailsModel? {
        val confirmData = preloadData.value?.signerParams?.confirmData ?: return null
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

        val preload = preloadData.value
        val signerParams = preload?.signerParams

        try {
            if (signerParams == null) {
                error("confirm input is not loaded")
            }
            val amount = when (val calculated = preload.amount) {
                is GemTransferAmountResult.Amount -> calculated.amount.value
                is GemTransferAmountResult.Error -> throw calculated.error
            }
            val transactionHash = execute(signerParams.copy(finalAmount = amount))
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

    private suspend fun execute(signerParams: SignerParams): String {
        val result = confirmService.execute(
            confirm = signerParams.confirmData,
            value = signerParams.finalAmount,
            networkFee = signerParams.fee.amount,
            simulation = simulationResult.value?.toJson(),
        )
        return when (result) {
            is GemExecuteResult.Signed -> result.data.first()
            is GemExecuteResult.Sent -> result.hashes.last()
        }
    }

    private data class ConfirmContent(
        val currency: Currency,
        val metadata: GemConfirmMetadata?,
        val feeAsset: Asset,
        val simulation: GemConfirmSimulationState,
    ) {
        val feeAssetUIModel: FeeAssetUIModel? = metadata?.let {
            FeeAssetUIModel.from(feeAsset, it.feeAssetBalance, it.feePrice(), currency)
        }

        fun assetPrice(asset: Asset): AssetPriceValue = metadata?.prices.orEmpty().toAssetPriceValue(asset, currency)
    }

    private data class Preload(
        val signerParams: SignerParams,
        val amount: GemTransferAmountResult,
        val content: ConfirmContent,
        val feeAssets: List<FeeAssetUIModel>,
    )

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
        val fromAsset = content.assetPrice(transfer.inputType.asset)
        val toAsset = transfer.inputType.toAsset?.let(content::assetPrice) ?: return null
        val summary = GemSwapQuoteSummary(swapData.quote.toJson())

        val provider = SwapProviderUIModelFactory.create(
            providerId = swapData.providerId,
            title = swapData.quote.providerData.protocolName,
            receiveAsset = toAsset,
            toValue = BigInteger(swapData.quote.toValue),
        )
        val model = SwapDetailsUIModelFactory.create(
            SwapDetailsUIModelInput(
                payAsset = fromAsset,
                receiveAsset = toAsset,
                fromValue = transfer.value,
                toValue = BigInteger(swapData.quote.toValue),
                provider = provider,
                slippageBps = swapData.quote.slippageBps,
                selectedSlippage = swapData.quote.slippageBps,
                etaInSeconds = swapData.quote.etaInSeconds,
                isProviderSelectable = false,
                priceImpact = fromAsset.swapValue(transfer.value)
                    .priceImpact(toAsset.swapValue(BigInteger(swapData.quote.toValue)))
                    ?.decodeJson(),
                minReceiveValue = summary.minReceiveValue(),
                etaMinutes = summary.etaMinutes(),
            ),
        ) ?: return null

        return ConfirmDetailElement.SwapDetails(model)
    }

    fun acquireFlow(asset: Asset): GemAcquireAssetFlow = confirmService.acquireAssetFlow(asset.chain.string)
}

