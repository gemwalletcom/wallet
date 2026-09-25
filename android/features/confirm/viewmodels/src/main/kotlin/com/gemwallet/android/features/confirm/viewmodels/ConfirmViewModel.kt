package com.gemwallet.android.features.confirm.viewmodels

import android.content.Context
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.gemstone.connection.ConnectionStatusObserver
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.domains.confirm.FeeAssetUIModel
import com.gemwallet.android.domains.confirm.FeeDetailsModel
import com.gemwallet.android.domains.confirm.FeeUIModel
import com.gemwallet.android.domains.confirm.applicationMetadata
import com.gemwallet.android.domains.confirm.asset
import com.gemwallet.android.domains.confirm.nftAsset
import com.gemwallet.android.domains.confirm.pack
import com.gemwallet.android.domains.confirm.perpetualType
import com.gemwallet.android.domains.confirm.swapData
import com.gemwallet.android.domains.confirm.toAsset
import com.gemwallet.android.domains.confirm.toFeeAssetUIModel
import com.gemwallet.android.domains.confirm.unpackTransferData
import com.gemwallet.android.ext.toAssetPriceInfo
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.confirm.models.ConfirmDetailElement
import com.gemwallet.android.features.confirm.viewmodels.localization.broadcastLabel
import com.gemwallet.android.features.confirm.viewmodels.localization.label
import com.gemwallet.android.features.confirm.viewmodels.localization.text
import com.gemwallet.android.features.confirm.viewmodels.models.AcquireAssetRequest
import com.gemwallet.android.features.confirm.viewmodels.models.AcquireOptionUIModel
import com.gemwallet.android.features.confirm.viewmodels.models.ConfirmErrorUIModel
import com.gemwallet.android.features.confirm.viewmodels.models.ConfirmHeaderUIModel
import com.gemwallet.android.features.confirm.viewmodels.models.ConfirmRowUIModel
import com.gemwallet.android.features.confirm.viewmodels.models.FeeSelectionUIModel
import com.gemwallet.android.features.confirm.viewmodels.models.acquireOptions
import com.gemwallet.android.features.confirm.viewmodels.models.confirmHeader
import com.gemwallet.android.features.confirm.viewmodels.models.feeItems
import com.gemwallet.android.features.confirm.viewmodels.models.listItem
import com.gemwallet.android.features.confirm.viewmodels.models.uiModel
import com.gemwallet.android.features.confirm.viewmodels.models.verificationListItem
import com.gemwallet.android.model.AssetPriceValue
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.infoSheet
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.actions.FinishConfirmAction
import com.gemwallet.android.ui.models.buttonState
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.models.swap.uiModel
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
import kotlinx.coroutines.flow.Flow
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
import uniffi.gemstone.GemAcquireAsset
import uniffi.gemstone.GemButtonState
import uniffi.gemstone.GemConfirmAction
import uniffi.gemstone.GemConfirmButton
import uniffi.gemstone.GemConfirmButtonKind
import uniffi.gemstone.GemConfirmException
import uniffi.gemstone.GemConfirmFeeRow
import uniffi.gemstone.GemConfirmFeeSelection
import uniffi.gemstone.GemConfirmLoad
import uniffi.gemstone.GemConfirmLoadOptions
import uniffi.gemstone.GemConfirmPhase
import uniffi.gemstone.GemConfirmScreen
import uniffi.gemstone.GemConfirmStage
import uniffi.gemstone.GemConfirmTransferServiceInterface
import uniffi.gemstone.GemConfirmation
import uniffi.gemstone.GemInfoAction
import uniffi.gemstone.GemRefreshKind
import uniffi.gemstone.GemSubmitResult
import uniffi.gemstone.GemTransferAmountResult
import uniffi.gemstone.GemTransferData
import uniffi.gemstone.PerpetualProvider
import uniffi.gemstone.PerpetualType
import uniffi.gemstone.SimulationResult
import uniffi.gemstone.TransactionInputType
import uniffi.gemstone.perpetualConfirmDetails
import uniffi.gemstone.swapQuoteDetails
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
    val isVerificationVisible = MutableStateFlow(false)
    val isVerificationFailed = MutableStateFlow(false)
    val verificationBridge = PaymentVerificationBridge(::onPaymentVerified, ::onPaymentVerificationFailed)
    private val loadOptions = MutableStateFlow<GemConfirmLoadOptions?>(null)
    private val requestSimulation = MutableStateFlow<SimulationResult?>(null)
    private val requestWallet = MutableStateFlow<Wallet?>(null)

    private val request = savedStateHandle.getStateFlow<String?>(RouteArgument.Params.key, null)
        .filterNotNull()
        .mapNotNull { paramsPack -> unpackTransferData(paramsPack) }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val session = getSession()
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val wallet = combine(requestWallet, session) { requestWallet, session -> requestWallet ?: session?.wallet }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val confirmation = combine(request.filterNotNull(), wallet.filterNotNull()) { request, wallet ->
        confirmService.confirmation(wallet.toGem(), request, requestSimulation.value).also {
            screen.value = it.screen()
            loadOptions.value = it.loadOptions()
        }
    }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val currency = confirmation.filterNotNull()
        .map { it.getCurrency().toPrimitives() }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val load = combine(
        confirmation.filterNotNull(),
        loadOptions.filterNotNull(),
        reload.onStart { emit(Unit) },
    ) { session, options, _ ->
        session to options
    }
        .transformLatest { (session, options) ->
            screen.update { it.onLoadStarted() }
            try {
                emit(session.state())
                val load = session.load(options)
                emit(load)
                screen.update { it.onLoaded(load) }
                isErrorSheetVisible.value = screen.value.presentsSheet()
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

    val viewState = combine(confirmation.filterNotNull(), screen, load) { confirmation, screen, _ ->
        confirmation.viewState(screen)
    }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val transfer = combine(request, load) { request, load -> load?.transfer ?: request }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val title = viewState.map { it?.title }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val isExternalRequest = transfer.map { it?.inputType?.applicationMetadata != null || it?.inputType is TransactionInputType.Payment }
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    val verification = viewState.map { it?.verification }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val simulation = content
        .map { it?.load?.simulation?.toSimulation(context) ?: Simulation() }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, Simulation())

    val simulationWarnings = viewState.map { it?.simulationWarnings.orEmpty() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val button = viewState.map { it?.button ?: GemConfirmButton(GemConfirmButtonKind.CONFIRM, GemButtonState.LOADING) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, GemConfirmButton(GemConfirmButtonKind.CONFIRM, GemButtonState.LOADING))

    val feeAsset = content.map { it?.feeAssetUIModel }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val feeAssets = content.map { it?.feeAssets.orEmpty() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val showsFeeAssets = content.map { it?.load?.showsFeeAssets() == true }
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    val detailElements = combine(transfer, content, ::buildDetailElements)
        .distinctUntilChanged()
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val transactionRows: StateFlow<List<ConfirmRowUIModel>> = combine(content, viewState) { content, viewState ->
        content ?: return@combine emptyList()
        viewState?.rowContents.orEmpty().mapNotNull { it.uiModel(context) }
    }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val feeInfo: StateFlow<FeeUIModel.FeeInfo?> = content.map { content ->
        val fee = content?.load?.fee ?: return@map null
        FeeUIModel.FeeInfo(
            amount = fee.value,
            additionalFees = fee.additionalFees,
            feeAsset = content.feeAssetUIModel.asset,
            price = content.load.metadata.feePrice()?.price,
            currency = content.currency,
            priority = fee.selectedPriority.toPrimitives(),
            display = fee.formatted,
        )
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val feeUIModel = combine(feeInfo, viewState) { feeInfo, viewState ->
        when (val feeRow = viewState?.feeRow ?: GemConfirmFeeRow.Loading) {
            GemConfirmFeeRow.Loading -> FeeUIModel.Calculating
            is GemConfirmFeeRow.Unavailable -> FeeUIModel.Unavailable(feeRow.text)
            GemConfirmFeeRow.Ready -> feeInfo ?: FeeUIModel.Calculating
        }
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val acquireRequestState = MutableStateFlow<AcquireAssetRequest?>(null)
    val acquireRequest = acquireRequestState.asStateFlow()

    val notice = viewState.map { it?.notice }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val loadError = combine(screen, confirmation, notice) { screen, confirmation, notice ->
        if (notice != null) return@combine null
        val error = screen.failure?.takeIf { it.stage == GemConfirmStage.LOAD }?.error ?: return@combine null
        ConfirmErrorUIModel(
            text = error.display().text(context),
            info = confirmation?.errorInfo(error)?.infoSheet { action -> (action as? GemInfoAction.Acquire)?.let { acquire(it.asset.toPrimitives(), it.acquire) } },
        )
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    fun acquire(asset: Asset, acquire: GemAcquireAsset) {
        acquireRequestState.value = AcquireAssetRequest(asset = asset, acquire = acquire)
    }

    fun dismissAcquire() = acquireRequestState.update { null }

    fun init(transfer: GemTransferData, simulationResult: SimulationResult? = null, wallet: Wallet? = null) {
        requestSimulation.value = simulationResult
        requestWallet.value = wallet
        viewModelScope.launch(ioDispatcher) {
            val pack = transfer.pack()
            if (savedStateHandle.get<String?>(RouteArgument.Params.key) == pack) {
                return@launch
            }
            screen.update { it.onLoadStarted() }
            savedStateHandle[RouteArgument.Params.key] = pack
        }
    }

    fun dismissErrorSheet() {
        isErrorSheetVisible.value = false
    }

    fun showVerification() {
        isVerificationVisible.value = true
    }

    fun dismissVerification() {
        isVerificationVisible.value = false
    }

    private fun onPaymentVerified() {
        isVerificationVisible.value = false
        fetch()
    }

    private fun onPaymentVerificationFailed() {
        isVerificationVisible.value = false
        isVerificationFailed.value = true
    }

    fun dismissVerificationError() {
        isVerificationFailed.value = false
    }

    private fun showError(error: Throwable) {
        screen.update { it.onLoadFailed(error.toConfirmError()) }
        isErrorSheetVisible.value = screen.value.presentsSheet()
    }

    val feeListItem: StateFlow<ListItemModel?> = combine(feeUIModel, feeAsset, showsFeeAssets, verification) { fee, asset, showsFeeAssets, verification ->
        verification?.let { verificationListItem(context) } ?: fee?.listItem(context, asset?.asset, showsFeeAssetSymbol = showsFeeAssets)
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val feeItems: StateFlow<List<ListItemModel>> = feeInfo.map { it?.feeItems(context).orEmpty() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val balanceChangeRows: StateFlow<List<ListItemModel>> = simulation.map { it.balanceChanges.map { change -> change.listItem() } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val acquireOptions: StateFlow<List<AcquireOptionUIModel>> = acquireRequest.map { request -> request?.let { acquireOptions(context, it) }.orEmpty() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val executeErrorText: StateFlow<String?> = screen.map { it.failure?.takeIf { failure -> failure.stage == GemConfirmStage.EXECUTE }?.error?.broadcastLabel(context) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val buttonLabel: StateFlow<String> = button.map { it.kind.label(context) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, context.getString(R.string.transfer_confirm))

    val buttonState: StateFlow<ButtonState> = button.map { it.state.buttonState() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, ButtonState.Loading)

    val header: StateFlow<ConfirmHeaderUIModel?> = combine(confirmation, load, screen) { confirmation, _, screen ->
        confirmation?.let { confirmHeader(it.header(screen), context) }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val feeSelectionUIModel: StateFlow<FeeSelectionUIModel> = loadOptions.filterNotNull()
        .map { FeeSelectionUIModel(it.feeSelection.selectedPriority()?.toPrimitives(), it.feeSelection.customGasPrice()) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, FeeSelectionUIModel(FeePriority.Normal, null))

    fun changeFeePriority(priority: FeePriority) = changeFeeSelection(GemConfirmFeeSelection.Priority(priority.toGem()))

    fun changeCustomFee(gasPrice: BigInteger) = changeFeeSelection(GemConfirmFeeSelection.Custom(gasPrice))

    private fun changeFeeSelection(selection: GemConfirmFeeSelection) = loadOptions.update { it?.onFeeSelection(selection) }

    fun feeDetailsModel(currentFee: FeeUIModel.FeeInfo): FeeDetailsModel? {
        val rows = viewState.value?.feeRates ?: return null
        return FeeDetailsModel(currentFee, rows)
    }

    fun changePaymentAsset(assetId: AssetId) {
        val current = transfer.value ?: return
        loadOptions.update { it?.onPaymentAsset(assetId.toIdentifier(), current) }
    }

    fun changeFeeAsset(assetId: AssetId) = loadOptions.update { it?.onFeeAsset(assetId.toIdentifier(), feeAsset.value?.asset?.id?.toIdentifier()) }

    fun fetch() {
        screen.update { it.onLoadStarted() }
        reload.tryEmit(Unit)
    }

    fun action(): GemConfirmAction? = screen.value.action()

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
            val (transactionHash, message) = when (val result = withContext(ioDispatcher) { session.submit() }) {
                is GemSubmitResult.Signed -> result.data.first() to result.message
                is GemSubmitResult.Sent -> result.hashes.last() to result.message
            }
            finishAction(transactionHash, message?.text(context))
        } catch (error: CancellationException) {
            throw error
        } catch (_: GemConfirmException.Cancelled) {
            screen.update { it.onExecuteCancelled() }
        } catch (err: Throwable) {
            screen.update { it.onExecuteFailed(err.toConfirmError()) }
        }
    }

    private data class ConfirmContent(val session: GemConfirmation, val currency: Currency, val load: GemConfirmLoad) {
        val feeAssetUIModel: FeeAssetUIModel = FeeAssetUIModel(load.feeAsset.toPrimitives(), load.feeAssetRow(currency.toGem()))

        val feeAssets: List<FeeAssetUIModel> = load.feeAssets.map { it.toFeeAssetUIModel() }

        fun assetPrice(asset: Asset): AssetPriceValue = AssetPriceValue(asset, load.metadata.price(asset.id.toIdentifier())?.toAssetPriceInfo(currency))
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
        val model = swapQuoteDetails(swapData.quote, fromAsset.asset.toGem(), toAsset.asset.toGem(), fromAsset.price?.price?.price, toAsset.price?.price?.price, content.currency.toGem())
            .uiModel() ?: return null

        return ConfirmDetailElement.SwapDetails(model)
    }
}

private fun Throwable.toConfirmError(): GemConfirmException = this as? GemConfirmException ?: GemConfirmException.Load(msg = message.orEmpty())
