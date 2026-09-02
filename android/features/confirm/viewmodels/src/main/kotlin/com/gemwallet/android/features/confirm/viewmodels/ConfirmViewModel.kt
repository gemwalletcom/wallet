package com.gemwallet.android.features.confirm.viewmodels

import com.wallet.core.primitives.swap.SwapPriceImpact

import com.gemwallet.android.domains.asset.swapValue
import android.util.Log
import com.gemwallet.android.ui.R
import uniffi.gemstone.GemTransferAmountResult
import uniffi.gemstone.GemAmountException
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
import com.gemwallet.android.application.confirm.cases.ConfirmTransaction
import com.gemwallet.android.application.confirm.cases.GetFeeAssets
import com.gemwallet.android.application.assets.cases.GetAssetInfo
import com.gemwallet.android.application.assets.cases.GetWalletAssets
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.blockchain.services.SignerPreloaderProxy
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.AssetInfo
import uniffi.gemstone.GemConfirmInput
import uniffi.gemstone.GemConfirmTransferService
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
import com.gemwallet.android.features.confirm.models.ConfirmDetailElement
import com.gemwallet.android.features.confirm.models.PerpetualModifyAutocloseFactory
import com.gemwallet.android.domains.confirm.ConfirmError
import com.gemwallet.android.domains.confirm.ConfirmState
import com.gemwallet.android.domains.confirm.toConfirmError
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
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
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
    private val getWalletAssets: GetWalletAssets,
    private val getAssetInfo: GetAssetInfo,
    private val signerPreloader: SignerPreloaderProxy,
    private val getFeeAssets: GetFeeAssets,
    private val confirmTransaction: ConfirmTransaction,
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

    private val simulationState = combine(request.filterNotNull(), simulationResult) { request, result ->
        request.transfer.inputType to result
    }
        .distinctUntilChanged()
        .map { (inputType, result) -> confirmService.simulationState(inputType, result?.toJson()) }
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val simulation = combine(simulationState, simulationResult, request) { state, result, params ->
        state?.toSimulation(
            warnings = result?.warnings.orEmpty(),
            chain = params?.transfer?.inputType?.asset?.id?.chain,
            confirmService = confirmService,
        ) ?: Simulation()
    }.stateIn(viewModelScope, SharingStarted.Eagerly, Simulation())

    val payloadAddressNames = simulationState
        .map { state ->
            state?.addressNames.orEmpty()
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

    private val assetsInfo = request.filterNotNull().mapNotNull {
        val inputType = it.transfer.inputType
        listOfNotNull(inputType.asset.id, inputType.toAsset?.id)
    }
    .flatMapLatest { getWalletAssets(it) }
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val feeAssets = request
        .filterNotNull()
        .map { it.transfer.inputType.asset.id.chain }
        .distinctUntilChanged()
        .flatMapLatest(getFeeAssets::invoke)
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    private val preloadData = combine(
        session,
        request.filterNotNull(),
        feeSelection,
        feeAssetSelection,
        restart,
    ) { session, request, feeSelection, feeAssetSelection, _ ->
        state.update { ConfirmState.Prepare }
        val owner = session?.wallet?.getAccount(request.transfer.inputType.asset.id.chain)
        if (owner == null) {
            state.update { ConfirmState.FatalError(R.string.errors_wallet_account_missing) }
            return@combine null
        }

        val preload = try {
            val result = signerPreloader.preload(
                walletId = session.wallet.id.id,
                input = request,
                selection = feeSelection,
                feeAssetSelection = feeAssetSelection,
            )
            result.simulation?.let { simulationResult.value = it }
            result
        } catch (error: CancellationException) {
            throw error
        } catch (err: Throwable) {
            state.update {
                ConfirmState.Error(err.toPreloadConfirmError())
            }
            return@combine null
        }

        state.update { ConfirmState.Ready }

        preload
    }
    .flowOn(Dispatchers.IO)
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val feeAssetInfo = preloadData.flatMapLatest { preload ->
        if (preload == null) {
            flowOf(null)
        } else {
            getAssetInfo(preload.signerParams.fee.feeAssetId)
        }
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val transferAmount = combine(preloadData, assetsInfo) { preload, assetsInfo ->
        if (preload == null) return@combine null
        val assetInfo = assetsInfo?.getByAssetId(preload.signerParams.input.transfer.inputType.asset.id) ?: return@combine null
        when (val amount = preload.amount) {
            is GemTransferAmountResult.Amount -> BigInteger(amount.amount.value)
            is GemTransferAmountResult.Error -> {
                state.update { ConfirmState.Error(amount.error.toConfirmError(amount.asset.toPrimitives())) }
                null
            }
        }
    }
    .flowOn(Dispatchers.IO)
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val amountUIModel = combine(request, assetsInfo, transferAmount) { request, assetsInfo, transferAmount ->
        val inputType = request?.transfer?.inputType ?: return@combine null
        val fromAssetId = inputType.asset.id
        val assetInfo = assetsInfo?.getByAssetId(fromAssetId) ?: return@combine null
        val toAssetInfo = inputType.toAsset?.let { assetsInfo.getByAssetId(it.id) ?: return@combine null }

        val amount = when {
            transferAmount != null -> Crypto(transferAmount)
            request.transfer.useMaxAmount -> return@combine null
            else -> Crypto(request.transfer.value.toBigInteger())
        }

        AmountUIModel(
            transactionType = inputType.transactionType().decodeJson<TransactionType>(),
            amount = amount.atomicValue,
            asset = assetInfo,
            fromAsset = assetInfo,
            fromAmount = amount.atomicValue.toString(),
            toAsset = toAssetInfo,
            toAmount = inputType.swapData?.quote?.toValue,
            nftAsset = inputType.nftAsset,
            price = assetInfo.price?.price?.price,
            currency = assetInfo.price?.currency ?: Currency.USD,
        )
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val perpetualType = request
        .map { it?.transfer?.inputType?.perpetualType }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val detailElements = combine(request, assetsInfo, ::buildDetailElements)
        .distinctUntilChanged()
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    private val recipientAddressName = request
        .filterNotNull()
        .map { it.transfer.inputType.asset.id.chain to it.transfer.recipient.address.takeIf { address -> address.isNotEmpty() } }
        .distinctUntilChanged()
        .map { (chain, address) ->
            address?.let { confirmService.addressName(chain.string, it)?.decodeJson<AddressName>() }
        }
        .flowOn(Dispatchers.IO)

    val transactionProperties = combine(request, session, assetsInfo, recipientAddressName) { request, session, assetsInfo, addressName ->
        request ?: return@combine emptyList()
        session ?: return@combine emptyList()
        assetsInfo ?: return@combine emptyList()
        buildConfirmProperties(request.transfer, session.wallet, assetsInfo, addressName)
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val feeUIModel = combine(preloadData, feeAssetInfo, state) { preload, feeAssetInfo, state ->
        val signerParams = preload?.signerParams
        val amount = signerParams?.fee?.amount
        val result = if (state is ConfirmState.Prepare) {
            FeeUIModel.Calculating
        } else if (amount == null || feeAssetInfo == null) {
            if (state is ConfirmState.Error) FeeUIModel.Error else FeeUIModel.Calculating
        } else {
            FeeUIModel.FeeInfo(
                amount = amount,
                feeAsset = feeAssetInfo.asset,
                price = feeAssetInfo.price?.price?.price,
                currency = feeAssetInfo.price?.currency ?: Currency.USD,
                priority = signerParams.fee.priority,
            )
        }
        result
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val feeValue = feeUIModel.map { (it as? FeeUIModel.FeeInfo)?.cryptoAmountWithFiat.orEmpty() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, "")

    val feeRates = preloadData.map { it?.signerParams?.feeRates.orEmpty() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    fun init(input: GemConfirmInput, simulationResult: SimulationResult? = null) {
        this.simulationResult.value = simulationResult
        feeSelection.value = FeeSelection.Preset(input.transfer.inputType.defaultFeePriority().toPrimitives())
        viewModelScope.launch(Dispatchers.IO) {
            val pack = transferService.pack(input)
            if (savedStateHandle.get<String?>(RouteArgument.Params.key) == pack) {
                return@launch
            }
            state.update { ConfirmState.Prepare }
            savedStateHandle[RouteArgument.Params.key] = pack
        }
    }

    fun feeDetailsModel(
        currentFee: FeeUIModel.FeeInfo,
        feeAssetInfo: AssetInfo,
        feeRates: List<GemFeeRate>,
        unitSymbol: String,
    ): FeeDetailsModel = FeeDetailsModel.from(currentFee, feeAssetInfo, feeRates, unitSymbol)

    fun changeFeeSelection(selection: FeeSelection) {
        if (selection == feeSelection.value) return
        state.update { ConfirmState.Prepare }
        feeSelection.update { selection }
    }

    fun changeFeeAsset(assetId: AssetId) {
        if (feeAssetInfo.value?.asset?.id == assetId) return
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
        val assetInfo = assetsInfo.value?.getByAssetId(signerParams?.input?.transfer?.inputType?.asset?.id ?: return@launch)
        val feeAssetInfo = feeAssetInfo.value
        val session = session.value

        try {
            if (assetInfo == null || assetInfo.owner == null || session == null || feeAssetInfo == null) {
                throw ConfirmError.TransactionIncorrect
            }
            val amount = when (val calculated = preload.amount) {
                is GemTransferAmountResult.Amount -> BigInteger(calculated.amount.value)
                is GemTransferAmountResult.Error -> throw calculated.error.toConfirmError(calculated.asset.toPrimitives())
            }
            val transactionHash = confirmTransaction(
                signerParams.copy(finalAmount = amount),
                session,
                assetInfo,
                viewModelScope,
                simulationResult.value,
            )
            state.update { ConfirmState.Result(transactionHash = transactionHash) }
            viewModelScope.launch(Dispatchers.Main) {
                finishAction(transactionHash)
            }
        } catch (error: CancellationException) {
            throw error
        } catch (err: Throwable) {
            state.update { ConfirmState.BroadcastError(err.toBroadcastConfirmError()) }
        }
    }

    private fun List<AssetInfo>.getByAssetId(assetId: AssetId): AssetInfo? {
        val str = assetId.toIdentifier()
        return firstOrNull { it.id().toIdentifier() == str }
    }

    private fun buildDetailElements(
        request: GemConfirmInput?,
        assetsInfo: List<AssetInfo>?,
    ): List<ConfirmDetailElement> {
        return listOfNotNull(
            buildSwapDetailElement(request?.transfer, assetsInfo),
            buildPerpetualDetailElement(request?.transfer?.inputType?.perpetualType),
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
        assetsInfo: List<AssetInfo>?,
    ): ConfirmDetailElement.SwapDetails? {
        val inputType = transfer?.inputType ?: return null
        val swapData = inputType.swapData ?: return null
        val assetsInfo = assetsInfo ?: return null
        val fromAssetInfo = assetsInfo.getByAssetId(inputType.asset.id) ?: return null
        val toAssetInfo = assetsInfo.getByAssetId(inputType.toAsset?.id ?: return null) ?: return null

        val provider = SwapProviderUIModelFactory.create(
            providerId = swapData.providerId,
            title = swapData.quote.providerData.protocolName,
            receiveAsset = toAssetInfo,
            toValue = swapData.quote.toValue,
        )
        val model = SwapDetailsUIModelFactory.create(
            SwapDetailsUIModelInput(
                payAsset = fromAssetInfo,
                receiveAsset = toAssetInfo,
                fromValue = transfer.value,
                toValue = swapData.quote.toValue,
                provider = provider,
                slippageBps = swapData.quote.slippageBps,
                selectedSlippage = swapData.quote.slippageBps,
                etaInSeconds = swapData.quote.etaInSeconds,
                isProviderSelectable = false,
                priceImpact = confirmService.swapPriceImpact(
                    fromAssetInfo.swapValue(transfer.value),
                    toAssetInfo.swapValue(swapData.quote.toValue),
                )?.decodeJson(),
            ),
        ) ?: return null

        return ConfirmDetailElement.SwapDetails(model)
    }

    private companion object {
        const val TAG = "Confirm"
    }

}

