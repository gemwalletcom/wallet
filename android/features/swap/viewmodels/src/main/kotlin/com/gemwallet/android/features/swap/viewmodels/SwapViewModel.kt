package com.gemwallet.android.features.swap.viewmodels

import android.content.Context
import android.util.Log
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.foundation.text.input.clearText
import androidx.compose.foundation.text.input.setTextAndPlaceCursorAtEnd
import androidx.compose.runtime.snapshotFlow
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.assets.cases.GetAssetInfo
import com.gemwallet.android.application.swap.cases.RequestSwapQuotes
import com.gemwallet.android.application.swap.cases.SwapQuoteRequestParams
import com.gemwallet.android.application.swap.cases.SwapQuotesResult
import com.gemwallet.android.application.swap.cases.toGem
import com.gemwallet.android.domains.asset.fiatEquivalent
import com.gemwallet.android.domains.confirm.ConfirmTransferInput
import com.gemwallet.android.domains.gemConfig
import com.gemwallet.android.domains.swap.SwapItemType
import com.gemwallet.android.domains.swap.toGem
import com.gemwallet.android.ext.GemConstants
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.features.swap.viewmodels.models.QuoteState
import com.gemwallet.android.features.swap.viewmodels.models.SwapUiState
import com.gemwallet.android.features.swap.viewmodels.models.createSwapUiState
import com.gemwallet.android.math.numberFormat
import com.gemwallet.android.math.parseInputNumberOrNull
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.text
import com.gemwallet.android.model.toAssetPriceValue
import com.gemwallet.android.model.toGem
import com.gemwallet.android.ui.components.swap.SlippageStateUIModel
import com.gemwallet.android.ui.components.swap.uiModel
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.models.swap.SwapDetailsUIModelFactory
import com.gemwallet.android.ui.models.swap.SwapDetailsUIModelInput
import com.gemwallet.android.ui.models.swap.SwapSlippage
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Currency
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.distinctUntilChangedBy
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.launchIn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemLocalizedText
import uniffi.gemstone.GemPercentageStyle
import uniffi.gemstone.GemSlippageSelection
import uniffi.gemstone.GemSlippageSession
import uniffi.gemstone.GemSwapButtonAction
import uniffi.gemstone.GemSwapPairSelection
import uniffi.gemstone.GemSwapQuoteInput
import uniffi.gemstone.GemSwapQuoteServiceInterface
import uniffi.gemstone.GemSwapRequest
import uniffi.gemstone.SwapProvider
import uniffi.gemstone.SwapperException
import uniffi.gemstone.availableBalanceText
import uniffi.gemstone.formattedPercentage
import uniffi.gemstone.newSlippageSession
import uniffi.gemstone.swapperQuoteSummary
import java.math.BigInteger
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class SwapViewModel @Inject constructor(
    private val getAssetInfo: GetAssetInfo,
    requestSwapQuotes: RequestSwapQuotes,
    private val savedStateHandle: SavedStateHandle,
    private val swapQuoteService: GemSwapQuoteServiceInterface,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    private val session = MutableStateFlow(swapQuoteService.newSession())

    val payValue: TextFieldState = TextFieldState()
    val receiveValue: TextFieldState = TextFieldState()

    private val payValueFlow = snapshotFlow { payValue.text }
        .map { it.toString() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, "")

    private val selectedSlippageBps = MutableStateFlow<UInt?>(null)
    val selectedSlippage: StateFlow<UInt?> = selectedSlippageBps.asStateFlow()

    private val slippageSession = MutableStateFlow<GemSlippageSession?>(null)
    val slippage: StateFlow<SlippageStateUIModel?> = slippageSession
        .map { it?.viewState()?.uiModel(context) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val refreshRequests = MutableSharedFlow<Unit>(extraBufferCapacity = 1)
    private val refreshEnabled = MutableStateFlow(false)
    private val quoteRefreshEnabled = combine(
        refreshEnabled,
        session.distinctUntilChangedBy { it.isTransferLoading() to it.refreshPausedUntilRestart },
    ) { isEnabled, quoteSession -> quoteSession.refreshesQuotes(isEnabled) }

    private val payAssetIdFlow = savedStateHandle.getStateFlow<String?>(RouteArgument.FromAssetId.key, null)
        .map { it?.toAssetId() }

    private val receiveAssetIdFlow = savedStateHandle.getStateFlow<String?>(RouteArgument.ToAssetId.key, null)
        .map { it?.toAssetId() }

    val payAsset = payAssetIdFlow
        .flatMapLatest { assetId -> assetId?.let { getAssetInfo(it) } ?: flow { emit(null) } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val receiveAsset = receiveAssetIdFlow
        .flatMapLatest { assetId -> assetId?.let { getAssetInfo(it) } ?: flow { emit(null) } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val payBalance: StateFlow<GemLocalizedText?> = payAsset.map { it?.availableBalance() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val receiveBalance: StateFlow<GemLocalizedText?> = receiveAsset.map { it?.availableBalance() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val quoteInput: StateFlow<GemSwapQuoteInput?> = session.map { it.input }
        .distinctUntilChanged()
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val payEquivalentFormatted = combine(payValueFlow, payAsset) { text, pay ->
        val value = text.parseInputNumberOrNull() ?: return@combine ""
        pay?.fiatEquivalent(Crypto(value, pay.asset.decimals).atomicValue) ?: ""
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, "")

    private val quoteRequestParams = combine(quoteInput, payAsset, receiveAsset) { input, pay, receive ->
        if (input == null || pay == null || receive == null) {
            null
        } else {
            SwapQuoteRequestParams(input, pay, receive)
        }
    }
        .distinctUntilChangedBy { it?.key }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val quoteResults = requestSwapQuotes(
        requestParams = quoteRequestParams,
        refreshRequests = refreshRequests,
        refreshEnabled = quoteRefreshEnabled,
        onFetchStarted = ::onQuoteFetchStarted,
        refreshIntervalMillis = GemConstants.swapQuoteRefreshInterval.inWholeMilliseconds,
        debounceMillis = GemConstants.swapQuoteDebounce.inWholeMilliseconds,
    )

    val quote = combine(session, payAsset, receiveAsset) { quoteSession, pay, receive ->
        val request = quoteSession.quotes?.request
        val selected = quoteSession.quote()
        if (request == null || selected == null || pay?.id()?.toIdentifier() != request.payAssetId || receive?.id()?.toIdentifier() != request.receiveAssetId) {
            null
        } else {
            QuoteState(selected, pay, receive)
        }
    }
        .distinctUntilChanged()
        .onEach { state -> setReceive(state?.let { session.value.receiveAmount()?.text() }.orEmpty()) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val providers = combine(session, quote) { quoteSession, current ->
        val receive = current?.receive ?: return@combine emptyList()
        quoteSession.providerRows(receive.asset.toGem(), receive.price?.price?.price, (receive.price?.currency ?: Currency.USD).toGem())
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val toEquivalentFormatted = quote.mapLatest { quote ->
        quote?.receive?.fiatEquivalent(quote.quote.toValue) ?: ""
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, "")

    private val viewState = combine(session, payAsset) { quoteSession, pay ->
        quoteSession.viewState(pay?.balance?.balance?.available ?: BigInteger.ZERO, pay?.asset?.toGem())
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val swapDetails = combine(quote, providers, viewState) { quote, providers, state ->
        if (quote == null) {
            return@combine null
        }
        val summary = swapperQuoteSummary(quote.quote, quote.pay.asset.toGem(), quote.receive.asset.toGem(), quote.pay.price?.price?.price, quote.receive.price?.price?.price)

        val provider = providers.firstOrNull { it.isSelected } ?: return@combine null

        SwapDetailsUIModelFactory.create(
            SwapDetailsUIModelInput(
                summary = summary,
                provider = provider,
                providers = providers,
                slippageBps = quote.quote.data.slippageBps,
                selectedSlippage = selectedSlippageBps.value,
                isProviderSelectable = state?.allowsProviderSelection ?: false,
            ),
        )
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val uiState = viewState.map { state -> state?.let { createSwapUiState(it, context) } ?: SwapUiState() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, SwapUiState())

    init {
        viewModelScope.launch {
            selectedSlippageBps.value = swapQuoteService.slippageBps()
        }
        combine(payValueFlow, payAsset, receiveAsset, selectedSlippageBps) { text, pay, receive, slippageBps ->
            session.update { it.onInputChanged(text, pay?.asset?.toGem(), receive?.asset?.toGem(), pay?.balance?.balance?.available ?: BigInteger.ZERO, slippageBps, numberFormat()) }
        }.launchIn(viewModelScope)
        quoteResults
            .onEach(::onQuoteResults)
            .launchIn(viewModelScope)
        combine(payAssetIdFlow, receiveAssetIdFlow) { pay, receive -> listOfNotNull(pay, receive).map { it.toIdentifier() } }
            .distinctUntilChanged()
            .onEach(::refreshPair)
            .launchIn(viewModelScope)
        viewModelScope.launch { suggestPair() }
    }

    private suspend fun suggestPair() {
        if (savedStateHandle.get<String?>(RouteArgument.ToAssetId.key) != null) {
            return
        }
        val payAssetId = savedStateHandle.get<String?>(RouteArgument.FromAssetId.key)
        val suggestion = swapQuoteService.suggestPair(payAssetId) ?: return
        savedStateHandle[RouteArgument.FromAssetId.key] = suggestion.payAssetId
        savedStateHandle[RouteArgument.ToAssetId.key] = suggestion.receiveAssetId
    }

    fun onSelect(type: SwapItemType, assetId: AssetId) {
        val selection = swapQuoteService.selectPairAsset(
            GemSwapPairSelection(
                payAssetId = payAsset.value?.id()?.toIdentifier(),
                receiveAssetId = receiveAsset.value?.id()?.toIdentifier(),
            ),
            type.toGem(),
            assetId.toIdentifier(),
        )
        val payChanged = selection.payAssetId != payAsset.value?.id()?.toIdentifier()
        savedStateHandle[RouteArgument.FromAssetId.key] = selection.payAssetId
        savedStateHandle[RouteArgument.ToAssetId.key] = selection.receiveAssetId
        if (payChanged) {
            payValue.clearText()
        }
    }

    fun switchSwap() = viewModelScope.launch {
        val payAssetId = payAsset.value?.id()?.toIdentifier()
        val receiveAssetId = receiveAsset.value?.id()?.toIdentifier()
        savedStateHandle[RouteArgument.FromAssetId.key] = receiveAssetId
        savedStateHandle[RouteArgument.ToAssetId.key] = payAssetId
        payValue.clearText()
    }

    fun setProvider(provider: SwapProvider) {
        session.update { it.onProviderSelected(provider) }
    }

    fun openSlippage() {
        val chain = payAsset.value?.asset?.id?.chain ?: return
        val selection = selectedSlippageBps.value?.let { GemSlippageSelection.Manual(it) } ?: GemSlippageSelection.Auto
        slippageSession.value = newSlippageSession(selection, chain.string, SwapSlippage.numberFormat())
    }

    fun onSlippageAuto(isAuto: Boolean) {
        slippageSession.update { it?.onAuto(isAuto) }
    }

    fun onSlippageInput(text: String) {
        slippageSession.update { it?.onInput(text) }
    }

    fun closeSlippage() {
        val state = slippageSession.value?.viewState() ?: return
        slippageSession.value = null
        if (state.allowsConfirm) {
            setSlippage((state.selection as? GemSlippageSelection.Manual)?.bps)
        }
    }

    private fun setSlippage(slippageBps: UInt?) {
        if (slippageBps == selectedSlippageBps.value) {
            return
        }
        selectedSlippageBps.update { slippageBps }
        viewModelScope.launch(ioDispatcher) {
            runCatchingCancellable { swapQuoteService.setSlippageBps(slippageBps) }
                .onFailure { Log.e(TAG, "saving the slippage failed", it) }
        }
    }

    fun onSelectPercent(percent: Int) {
        val asset = payAsset.value ?: return
        val value = swapQuoteService.amountForPercent(asset.balance.balance.available, percent.toUInt())
        val text = numberFormat().inputText(value.toString(), asset.asset.decimals.toUInt()) ?: return
        payValue.clearText()
        payValue.setTextAndPlaceCursorAtEnd(text)
    }

    fun refresh() {
        val params = quoteRequestParams.value ?: return
        session.update { it.onRefreshRequested(params.key) }
        refreshRequests.tryEmit(Unit)
    }

    fun onPrimaryAction(onConfirm: (ConfirmTransferInput) -> Unit, onShowPriceImpactWarning: () -> Unit, authorize: (() -> Unit) -> Unit) {
        val state = uiState.value
        if (state.buttonState != ButtonState.Enabled) {
            return
        }
        when (val action = viewState.value?.buttonAction ?: return) {
            GemSwapButtonAction.Swap -> {
                if (swapDetails.value?.shouldShowPriceImpactWarning == true) {
                    onShowPriceImpactWarning()
                } else {
                    authorize { swap(onConfirm) }
                }
            }

            GemSwapButtonAction.RetryTransfer -> authorize { swap(onConfirm) }

            GemSwapButtonAction.RetryQuote -> refresh()

            is GemSwapButtonAction.UseMinimumAmount -> setPayValue(action.value)

            GemSwapButtonAction.InsufficientBalance -> Unit
        }
    }

    fun setRefreshEnabled(isEnabled: Boolean) {
        if (isEnabled && !refreshEnabled.value) {
            session.update { it.onRefreshResumed() }
        }
        refreshEnabled.value = isEnabled
    }

    fun swap(onConfirm: (ConfirmTransferInput) -> Unit) = viewModelScope.launch(ioDispatcher) {
        val pending = quote.value ?: return@launch
        val started = session.value.startTransfer() ?: return@launch
        val transfer = started.transferPhase
        session.value = started

        try {
            val params = swapQuoteService.getTransfer(pending.quote)
                .transferData(pending.pay.asset.toGem(), pending.receive.asset.toGem())
            if (session.value.transferPhase != transfer) {
                return@launch
            }
            withContext(Dispatchers.Main) {
                onConfirm(ConfirmTransferInput(params))
            }
            session.update { it.onTransferHandedOff(transfer) }
        } catch (err: SwapperException) {
            session.update { it.onTransferFailed(transfer, err) }
        }
    }

    private suspend fun refreshPair(assetIds: List<String>) = withContext(ioDispatcher) {
        runCatchingCancellable { swapQuoteService.refreshPair(assetIds) }
            .getOrNull()
            ?.forEach { Log.e(TAG, "pair refresh failed at ${it.step}: ${it.message}") }
    }

    private fun onQuoteFetchStarted(requestKey: GemSwapRequest) {
        session.update { it.onFetchStarted(requestKey) }
    }

    private fun onQuoteResults(results: SwapQuotesResult?) {
        results ?: return
        session.update { it.onQuoteResults(results.toGem()) }
    }

    private fun setPayValue(amount: BigInteger) {
        val asset = payAsset.value?.asset ?: return
        val text = numberFormat().inputText(amount.toString(), asset.decimals.toUInt()) ?: return
        payValue.clearText()
        payValue.setTextAndPlaceCursorAtEnd(text)
    }

    private suspend fun setReceive(amount: String) = withContext(Dispatchers.Main) {
        receiveValue.clearText()
        receiveValue.setTextAndPlaceCursorAtEnd(amount)
    }

    companion object {
        val percentSuggestions = gemConfig.getSwapConfig().amountPercentPresets.map { formattedPercentage(it.toDouble(), GemPercentageStyle.UNSIGNED_COMPACT) }
    }
}

private const val TAG = "Swap"

private fun AssetInfo.availableBalance(): GemLocalizedText = availableBalanceText(asset.toGem(), balance.toGem())
