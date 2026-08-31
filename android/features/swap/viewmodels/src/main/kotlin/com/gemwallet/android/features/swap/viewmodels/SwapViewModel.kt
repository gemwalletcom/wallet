package com.gemwallet.android.features.swap.viewmodels

import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.foundation.text.input.clearText
import androidx.compose.foundation.text.input.setTextAndPlaceCursorAtEnd
import androidx.compose.runtime.snapshotFlow
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.assets.cases.EnableAsset
import com.gemwallet.android.application.swap.cases.BuildSwapConfirmParams
import com.gemwallet.android.application.swap.cases.RequestSwapQuotes
import com.gemwallet.android.application.swap.cases.SwapNoQuoteException
import com.gemwallet.android.application.swap.cases.SwapQuoteRequestKey
import com.gemwallet.android.application.swap.cases.SwapQuoteRequestParams
import com.gemwallet.android.application.swap.cases.SwapQuotesResult
import com.gemwallet.android.application.swap.cases.create
import com.gemwallet.android.application.swap.cases.matches
import com.gemwallet.android.application.assets.cases.GetAssetInfo
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.domains.asset.calculateFiat
import com.gemwallet.android.domains.asset.formatFiat
import com.gemwallet.android.domains.swap.SwapItemType
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.features.swap.viewmodels.models.SwapQuoteSession
import com.gemwallet.android.features.swap.viewmodels.models.SwapTransferPhase
import com.gemwallet.android.features.swap.viewmodels.models.SwapUiState
import com.gemwallet.android.features.swap.viewmodels.models.createSwapUiState
import com.gemwallet.android.features.swap.viewmodels.models.formattedToAmount
import com.gemwallet.android.features.swap.viewmodels.models.onFetchStarted
import com.gemwallet.android.features.swap.viewmodels.models.onProviderSelected
import com.gemwallet.android.features.swap.viewmodels.models.onQuoteInvalidated
import com.gemwallet.android.features.swap.viewmodels.models.onQuoteResults
import com.gemwallet.android.features.swap.viewmodels.models.onRefreshRequested
import com.gemwallet.android.features.swap.viewmodels.models.onRequestParamsChanged
import com.gemwallet.android.features.swap.viewmodels.models.onTransferAbandoned
import com.gemwallet.android.features.swap.viewmodels.models.onTransferFailed
import com.gemwallet.android.features.swap.viewmodels.models.onTransferHandedOff
import com.gemwallet.android.features.swap.viewmodels.models.receiveEquivalent
import com.gemwallet.android.features.swap.viewmodels.models.startTransfer
import com.gemwallet.android.math.multiplyByPercent
import com.gemwallet.android.math.parseInputNumberOrNull
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.models.swap.SwapDetailsUIModelFactory
import com.gemwallet.android.ui.models.swap.SwapDetailsUIModelInput
import com.gemwallet.android.ui.models.swap.SwapProviderUIModelFactory
import com.wallet.core.primitives.AssetId
import dagger.hilt.android.lifecycle.HiltViewModel
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
import uniffi.gemstone.Config
import uniffi.gemstone.GemSwapButtonAction
import uniffi.gemstone.GemSwapButtonInput
import uniffi.gemstone.GemStreamSubscriptionService
import uniffi.gemstone.GemSwapQuoteService
import uniffi.gemstone.GemSwapServiceInterface
import uniffi.gemstone.SwapperException
import uniffi.gemstone.SwapperProvider
import java.math.BigDecimal
import java.math.BigInteger
import javax.inject.Inject
import com.gemwallet.android.ext.runCatchingCancellable

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class SwapViewModel @Inject constructor(
    private val getSession: GetSession,
    private val getAssetInfo: GetAssetInfo,
    private val enableAsset: EnableAsset,
    private val buildSwapConfirmParams: BuildSwapConfirmParams,
    private val userConfig: UserConfig,
    private val swapService: GemSwapServiceInterface,
    requestSwapQuotes: RequestSwapQuotes,
    private val savedStateHandle: SavedStateHandle,
    private val swapQuoteService: GemSwapQuoteService,
    private val streamSubscriptionService: GemStreamSubscriptionService,
) : ViewModel() {

    private val session = MutableStateFlow(SwapQuoteSession())

    val payValue: TextFieldState = TextFieldState()
    val receiveValue: TextFieldState = TextFieldState()

    private val payValueFlow = snapshotFlow { payValue.text }
        .map { it.toString() }
        .map { it.parseInputNumberOrNull() ?: BigDecimal.ZERO }
        .stateIn(viewModelScope, SharingStarted.Eagerly, BigDecimal.ZERO)

    private val selectedSlippageBps = MutableStateFlow<UInt?>(null)
    val selectedSlippage: StateFlow<UInt?> = selectedSlippageBps.asStateFlow()

    val slippageWarningThresholdBps: UInt by lazy { Config().getSwapConfig().highSlippageWarningBps }

    private val refreshRequests = MutableSharedFlow<Unit>(extraBufferCapacity = 1)
    private val refreshEnabled = MutableStateFlow(false)
    private val transferPhase = session.map { it.transferPhase }.distinctUntilChanged()
    private val refreshPaused = session.map { it.refreshPausedUntilRestart }.distinctUntilChanged()
    private val quoteRefreshEnabled = combine(refreshEnabled, transferPhase, refreshPaused) { isEnabled, transferState, isPaused ->
            isEnabled && !isPaused && transferState !is SwapTransferPhase.Loading
        }

    val payAsset = savedStateHandle.getStateFlow<String?>(RouteArgument.FromAssetId.key, null)
        .map { it?.toAssetId() }
        .onEach { id -> id?.let { updateBalance(it) } }
        .flatMapLatest { assetId -> assetId?.let { getAssetInfo(it) } ?: flow { emit(null) } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val defaultSlippageBps: StateFlow<UInt?> = payAsset
        .map { asset -> asset?.let { swapQuoteService.defaultSlippage(it.asset.id.chain.string).bps } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val receiveAsset = savedStateHandle.getStateFlow<String?>(RouteArgument.ToAssetId.key, null)
        .map { it?.toAssetId() }
        .onEach { id -> id?.let { updateBalance(it) } }
        .flatMapLatest { assetId -> assetId?.let { getAssetInfo(it) } ?: flow { emit(null) } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val payEquivalentFormatted = combine(payValueFlow, payAsset) { input, fromAsset ->
            fromAsset?.let {
                val equivalentValue = it.calculateFiat(input)
                it.formatFiat(equivalentValue)
            } ?: ""
        }
        .stateIn(viewModelScope, SharingStarted.Eagerly, "")

    private val quoteRequestParams = combine(payValueFlow, payAsset, receiveAsset, selectedSlippageBps) { value, fromAsset, toAsset, slippageBps ->
            SwapQuoteRequestParams.create(value, fromAsset, toAsset, slippageBps)
        }
        .distinctUntilChangedBy { it?.key }
        .onEach(::onQuoteRequestParamsChanged)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val matchedQuoteResults = combine(
        quoteRequestParams,
        requestSwapQuotes(
            requestParams = quoteRequestParams,
            refreshRequests = refreshRequests,
            refreshEnabled = quoteRefreshEnabled,
            onFetchStarted = ::onQuoteFetchStarted,
            refreshIntervalMillis = swapQuoteService.refreshIntervalMilliseconds().toLong(),
        ),
    ) { params, results ->
        results?.takeIf { it.matches(params) }
    }

    val providers = session.map { it.quotes }
        .distinctUntilChanged()
        .mapLatest { quotes ->
            val quoteState = quotes ?: return@mapLatest emptyList()
            quoteState.items.map { item ->
                SwapProviderUIModelFactory.create(
                    provider = item.data.provider,
                    receiveAsset = quoteState.receive,
                    toValue = item.toValue,
                )
            }
        }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val quote = session.map { it.quote }
        .distinctUntilChanged()
        .onEach { state -> setReceive(state?.formattedToAmount ?: "") }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val toEquivalentFormatted = quote.mapLatest { quote ->
            quote?.receive?.formatFiat(quote.receiveEquivalent) ?: ""
        }
        .stateIn(viewModelScope, SharingStarted.Eagerly, "")

    val swapDetails = combine(quote, providers) { quote, providers ->
            if (quote == null) {
                return@combine null
            }

            val provider = providers.firstOrNull { item ->
                item.id == quote.quote.data.provider.id &&
                    item.title == quote.quote.data.provider.protocol
            } ?: SwapProviderUIModelFactory.create(
                provider = quote.quote.data.provider,
                receiveAsset = quote.receive,
                toValue = quote.quote.toValue,
            )

            SwapDetailsUIModelFactory.create(
                SwapDetailsUIModelInput(
                    payAsset = quote.pay,
                    receiveAsset = quote.receive,
                    fromValue = quote.quote.fromValue,
                    toValue = quote.quote.toValue,
                    provider = provider,
                    providers = providers,
                    slippageBps = quote.quote.data.slippageBps,
                    selectedSlippage = selectedSlippageBps.value,
                    etaInSeconds = quote.quote.etaInSeconds,
                    isProviderSelectable = providers.size > 1,
                ),
                swapQuoteService,
            )
        }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val uiState = combine(session, payValueFlow, payAsset) { quoteSession, value, pay ->
            createSwapUiState(quoteSession, buttonAction(quoteSession, value, pay))
        }
        .stateIn(viewModelScope, SharingStarted.Eagerly, SwapUiState())

    init {
        viewModelScope.launch {
            selectedSlippageBps.value = userConfig.swapSlippageBps().firstOrNull()
        }
        matchedQuoteResults
            .onEach(::onQuoteResults)
            .launchIn(viewModelScope)
        viewModelScope.launch { suggestPair() }
    }

    private suspend fun suggestPair() {
        if (savedStateHandle.get<String?>(RouteArgument.ToAssetId.key) != null) {
            return
        }
        val walletId = getSession().firstOrNull()?.wallet?.id ?: return
        val payAssetId = savedStateHandle.get<String?>(RouteArgument.FromAssetId.key)
        val suggestion = runCatchingCancellable { swapService.suggestPair(walletId.id, payAssetId) }.getOrNull() ?: return
        savedStateHandle[RouteArgument.FromAssetId.key] = suggestion.payAssetId
        savedStateHandle[RouteArgument.ToAssetId.key] = suggestion.receiveAssetId
    }

    fun onSelect(type: SwapItemType, assetId: AssetId) {
        session.update { it.onQuoteInvalidated() }
        when (type) {
            SwapItemType.Pay -> {
                if (receiveAsset.value?.id() == assetId) {
                    savedStateHandle[RouteArgument.ToAssetId.key] = null
                }
                savedStateHandle[RouteArgument.FromAssetId.key] = assetId.toIdentifier()
                payValue.clearText()
            }
            SwapItemType.Receive -> {
                if (payAsset.value?.id() == assetId) {
                    savedStateHandle[RouteArgument.FromAssetId.key] = null
                    payValue.clearText()
                }
                savedStateHandle[RouteArgument.ToAssetId.key] = assetId.toIdentifier()
            }
        }
    }

    fun switchSwap() = viewModelScope.launch {
        session.update { it.onQuoteInvalidated() }
        val payAssetId = payAsset.value?.id()?.toIdentifier()
        val receiveAssetId = receiveAsset.value?.id()?.toIdentifier()
        savedStateHandle[RouteArgument.FromAssetId.key] = receiveAssetId
        savedStateHandle[RouteArgument.ToAssetId.key] = payAssetId
        payValue.clearText()
    }

    fun setProvider(provider: SwapperProvider) {
        session.update { it.onProviderSelected(provider) }
    }

    fun setSlippage(slippageBps: UInt?) {
        if (slippageBps == selectedSlippageBps.value) {
            return
        }
        session.update { it.onQuoteInvalidated() }
        selectedSlippageBps.update { slippageBps }
        viewModelScope.launch(Dispatchers.IO) {
            userConfig.setSwapSlippageBps(slippageBps)
        }
    }

    fun onSelectPercent(percent: Int) {
        val asset = payAsset.value ?: return
        val value = asset.balance.balance.available.toBigInteger().multiplyByPercent(percent)
        payValue.clearText()
        payValue.setTextAndPlaceCursorAtEnd(
            Crypto(value).value(asset.asset.decimals).stripTrailingZeros().toPlainString()
        )
    }

    fun refresh() {
        val params = quoteRequestParams.value ?: return
        session.update { it.onRefreshRequested(params) }
        refreshRequests.tryEmit(Unit)
    }

    fun onPrimaryAction(
        onConfirm: (ConfirmParams) -> Unit,
        onShowPriceImpactWarning: () -> Unit,
        authorize: (() -> Unit) -> Unit,
    ) {
        val state = uiState.value
        if (state.buttonState != ButtonState.Enabled) {
            return
        }
        when (val action = state.buttonAction) {
            GemSwapButtonAction.Swap -> {
                if (swapDetails.value?.shouldShowPriceImpactWarning == true) {
                    onShowPriceImpactWarning()
                } else {
                    authorize { swap(onConfirm) }
                }
            }
            GemSwapButtonAction.RetryTransfer -> authorize { swap(onConfirm) }
            GemSwapButtonAction.RetryQuote -> refresh()
            is GemSwapButtonAction.UseMinimumAmount -> applyMinimumAmount(action.value)
            GemSwapButtonAction.InsufficientBalance -> Unit
        }
    }

    fun setRefreshEnabled(isEnabled: Boolean) {
        if (isEnabled && !refreshEnabled.value) {
            session.update {
                if (it.refreshPausedUntilRestart) it.copy(refreshPausedUntilRestart = false) else it
            }
        }
        refreshEnabled.value = isEnabled
    }

    fun swap(onConfirm: (ConfirmParams) -> Unit) = viewModelScope.launch(Dispatchers.IO) {
        val started = session.value.startTransfer()
        val transfer = started.second ?: return@launch
        val pending = started.first.quote ?: return@launch
        session.value = started.first

        try {
            val params = buildSwapConfirmParams(
                quote = pending.quote,
                pay = pending.pay,
                receive = pending.receive,
            ) ?: run {
                session.update { it.onTransferAbandoned(transfer) }
                return@launch
            }
            if (session.value.transferPhase != transfer) {
                return@launch
            }
            withContext(Dispatchers.Main) {
                onConfirm(params)
            }
            session.update { it.onTransferHandedOff(transfer) }
        } catch (err: SwapNoQuoteException) {
            session.update { it.onTransferFailed(transfer, err.cause ?: SwapperException.NoQuoteAvailable()) }
        } catch (err: Throwable) {
            session.update { it.onTransferFailed(transfer, err) }
        }
    }

    private fun updateBalance(id: AssetId) = viewModelScope.launch(Dispatchers.IO) {
        val currentSession = getSession().firstOrNull() ?: return@launch
        currentSession.wallet.getAccount(id.chain) ?: return@launch
        enableAsset(currentSession.wallet.id, id)
        runCatchingCancellable { streamSubscriptionService.addPrices(listOf(id.toIdentifier())) }
    }

    private fun onQuoteRequestParamsChanged(params: SwapQuoteRequestParams?) {
        session.update { it.onRequestParamsChanged(params) }
    }

    private fun onQuoteFetchStarted(requestKey: SwapQuoteRequestKey) {
        session.update { it.onFetchStarted(requestKey) }
    }

    private fun onQuoteResults(results: SwapQuotesResult?) {
        results ?: return
        session.update { it.onQuoteResults(results) }
    }

    private fun buttonAction(quoteSession: SwapQuoteSession, value: BigDecimal, pay: AssetInfo?): GemSwapButtonAction =
        swapQuoteService.buttonAction(
            GemSwapButtonInput(
                value = pay?.let { Crypto(value, it.asset.decimals).atomicValue.toString() } ?: "0",
                availableBalance = pay?.balance?.balance?.available ?: "0",
                quoteError = quoteSession.quoteError,
                transferError = quoteSession.transferError,
            )
        )

    private fun applyMinimumAmount(amount: String) {
        val asset = payAsset.value?.asset ?: return
        payValue.clearText()
        payValue.setTextAndPlaceCursorAtEnd(Crypto(BigInteger(amount)).value(asset.decimals).toString())
    }

    private suspend fun setReceive(amount: String) = withContext(Dispatchers.Main) {
        receiveValue.clearText()
        receiveValue.setTextAndPlaceCursorAtEnd(amount)
    }

    companion object {
        val percentSuggestions = Config().getSwapConfig().amountPercentPresets.map { it.toInt() }
    }
}
