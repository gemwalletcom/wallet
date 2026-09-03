package com.gemwallet.android.features.confirm.viewmodels

import com.gemwallet.android.ui.R
import uniffi.gemstone.GemTransferAmountResult
import uniffi.gemstone.GemConfirmException
import uniffi.gemstone.GemTransferData
import uniffi.gemstone.GemTransferService
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
import uniffi.gemstone.GemFeeRate
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.confirm.cases.BuildConfirmProperties
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.blockchain.services.confirmLoadOptions
import com.gemwallet.android.blockchain.services.toSignerParams
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.ext.toAssetPriceValue
import com.gemwallet.android.ext.toCurrency
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.AssetPriceValue
import uniffi.gemstone.GemConfirmSimulationState
import uniffi.gemstone.GemConfirmTransferService
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
import com.gemwallet.android.ui.models.hasCriticalWarning
import com.gemwallet.android.domains.confirm.AmountUIModel
import com.gemwallet.android.domains.confirm.FeeAssetUIModel
import com.gemwallet.android.domains.confirm.toFeeAssetUIModel
import com.gemwallet.android.features.confirm.models.ConfirmDetailElement
import com.gemwallet.android.features.confirm.models.PerpetualModifyAutocloseFactory
import com.gemwallet.android.domains.confirm.ConfirmState
import com.gemwallet.android.domains.confirm.FeeDetailsModel
import com.gemwallet.android.domains.confirm.FeeUIModel
import com.wallet.core.primitives.AddressName
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
    private val confirmService: GemConfirmTransferService,
    private val savedStateHandle: SavedStateHandle,
    private val transferService: GemTransferService,
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

    private val preloadData = combine(
        request.filterNotNull(),
        feeSelection,
        feeAssetSelection,
        restart,
    ) { request, feeSelection, feeAssetSelection, _ ->
        state.update { ConfirmState.Prepare }
        val input = try {
            confirmService.confirmInput(request)
        } catch (_: GemConfirmException.AccountMissing) {
            state.update { ConfirmState.FatalError(R.string.errors_wallet_account_missing) }
            return@combine null
        }

        val preload = try {
            val scene = confirmService.load(
                input = input,
                options = confirmLoadOptions(feeSelection, feeAssetSelection),
                simulation = simulationResult.value?.toJson(),
            )
            scene.preload.confirmData.simulation?.let { simulationResult.value = it.decodeJson() }
            val currency = confirmService.currency().toCurrency()
            Preload(
                signerParams = scene.preload.toSignerParams(),
                amount = scene.preload.amount,
                currency = currency,
                fromAsset = scene.preload.metadata.prices.toAssetPriceValue(request.inputType.asset, currency),
                toAsset = request.inputType.toAsset?.let { scene.preload.metadata.prices.toAssetPriceValue(it, currency) },
                feeAsset = FeeAssetUIModel.from(scene.preload.feeAsset.toPrimitives(), scene.preload.metadata.feeAssetBalance, scene.preload.metadata.prices, currency),
                feeAssets = scene.feeAssets.map { it.toFeeAssetUIModel(currency) },
                simulation = scene.simulation,
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

    val simulation = combine(preloadData, simulationResult, request) { preload, result, params ->
        preload?.simulation?.toSimulation(
            warnings = result?.warnings.orEmpty(),
            chain = params?.inputType?.asset?.id?.chain,
            confirmService = confirmService,
        ) ?: Simulation()
    }.stateIn(viewModelScope, SharingStarted.Eagerly, Simulation())

    val payloadAddressNames = preloadData
        .map { preload ->
            preload?.simulation?.addressNames.orEmpty()
                .map { it.decodeJson<AddressName>() }
                .filter { it.name.isNotEmpty() && !it.name.equals(it.address, ignoreCase = true) }
                .associate { it.address.lowercase() to it.name }
        }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyMap())

    val buttonState = combine(state, simulation) { state, simulation ->
        buttonState(
            enabled = state !is ConfirmState.Prepare
                && state !is ConfirmState.Sending
                && !simulation.warnings.hasCriticalWarning(),
            loading = state is ConfirmState.Sending || state is ConfirmState.Prepare || state is ConfirmState.Result,
        )
    }.stateIn(viewModelScope, SharingStarted.Eagerly, ButtonState.Loading)

    val feeAsset = preloadData.map { it?.feeAsset }
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

    val amountUIModel = combine(request, preloadData, transferAmount) { request, preload, transferAmount ->
        val inputType = request?.inputType ?: return@combine null
        preload ?: return@combine null

        val amount = when {
            transferAmount != null -> Crypto(transferAmount)
            request.useMaxAmount -> return@combine null
            else -> Crypto(request.value)
        }

        AmountUIModel(
            transactionType = inputType.transactionType().decodeJson<TransactionType>(),
            headerKind = inputType.headerKind(),
            amount = amount.atomicValue,
            fromAsset = preload.fromAsset,
            fromAmount = amount.atomicValue,
            toAsset = preload.toAsset,
            toAmount = inputType.swapData?.quote?.toValue?.let(::BigInteger),
            nftAsset = inputType.nftAsset,
            currency = preload.currency,
        )
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val perpetualType = request
        .map { it?.inputType?.perpetualType }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val detailElements = combine(request, preloadData, ::buildDetailElements)
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

    val feeUIModel = combine(preloadData, state) { preload, state ->
        val signerParams = preload?.signerParams
        val amount = signerParams?.fee?.amount
        if (state is ConfirmState.Prepare) {
            FeeUIModel.Calculating
        } else if (amount == null) {
            if (state is ConfirmState.Error) FeeUIModel.Error else FeeUIModel.Calculating
        } else {
            FeeUIModel.FeeInfo(
                amount = amount,
                feeAsset = preload.feeAsset.asset,
                price = preload.feeAsset.price?.price?.price,
                currency = preload.currency,
                priority = signerParams.fee.priority,
            )
        }
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val feeValue = feeUIModel.map { (it as? FeeUIModel.FeeInfo)?.cryptoAmountWithFiat.orEmpty() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, "")

    val feeRates = preloadData.map { it?.signerParams?.feeRates.orEmpty() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

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

    fun feeDetailsModel(
        currentFee: FeeUIModel.FeeInfo,
        feeAsset: FeeAssetUIModel,
        feeRates: List<GemFeeRate>,
        unitSymbol: String,
    ): FeeDetailsModel = FeeDetailsModel.from(currentFee, feeAsset, feeRates, unitSymbol)

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

    private data class Preload(
        val signerParams: SignerParams,
        val amount: GemTransferAmountResult,
        val currency: Currency,
        val fromAsset: AssetPriceValue,
        val toAsset: AssetPriceValue?,
        val feeAsset: FeeAssetUIModel,
        val feeAssets: List<FeeAssetUIModel>,
        val simulation: GemConfirmSimulationState,
    )

    private fun buildDetailElements(
        request: GemTransferData?,
        preload: Preload?,
    ): List<ConfirmDetailElement> {
        return listOfNotNull(
            buildSwapDetailElement(request, preload),
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
        preload: Preload?,
    ): ConfirmDetailElement.SwapDetails? {
        val swapData = transfer?.inputType?.swapData ?: return null
        val fromAsset = preload?.fromAsset ?: return null
        val toAsset = preload.toAsset ?: return null
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
}

