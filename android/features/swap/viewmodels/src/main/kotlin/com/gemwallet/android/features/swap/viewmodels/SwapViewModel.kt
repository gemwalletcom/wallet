package com.gemwallet.android.features.swap.viewmodels

import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.foundation.text.input.clearText
import androidx.compose.foundation.text.input.setTextAndPlaceCursorAtEnd
import androidx.compose.runtime.snapshotFlow
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.assets.coordinators.EnableAsset
import com.gemwallet.android.application.swap.coordinators.BuildSwapConfirmParams
import com.gemwallet.android.application.swap.coordinators.RequestSwapQuotes
import com.gemwallet.android.application.swap.coordinators.SwapNoQuoteException
import com.gemwallet.android.application.swap.coordinators.SwapQuoteRequestKey
import com.gemwallet.android.application.swap.coordinators.SwapQuoteRequestParams
import com.gemwallet.android.application.swap.coordinators.SwapQuotesResult
import com.gemwallet.android.application.swap.coordinators.create
import com.gemwallet.android.application.swap.coordinators.matches
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.config.UserConfig
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.domains.asset.calculateFiat
import com.gemwallet.android.domains.asset.formatFiat
import com.gemwallet.android.domains.swap.SwapItemType
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.features.swap.viewmodels.models.SwapActionState
import com.gemwallet.android.features.swap.viewmodels.models.SwapError
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
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.CurrencyFormatter
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
import uniffi.gemstone.SwapperProvider
import java.math.BigDecimal
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class SwapViewModel @Inject constructor(
    private val sessionRepository: SessionRepository,
    private val assetsRepository: AssetsRepository,
    private val enableAsset: EnableAsset,
    private val buildSwapConfirmParams: BuildSwapConfirmParams,
    private val userConfig: UserConfig,
    requestSwapQuotes: RequestSwapQuotes,
    private val savedStateHandle: SavedStateHandle,
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
        .flatMapLatest { assetId -> assetId?.let { assetsRepository.getAssetInfo(it) } ?: flow { emit(null) } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val receiveAsset = savedStateHandle.getStateFlow<String?>(RouteArgument.ToAssetId.key, null)
        .map { it?.toAssetId() }
        .onEach { id -> id?.let { updateBalance(it) } }
        .flatMapLatest { assetId -> assetId?.let { assetsRepository.getAssetInfo(it) } ?: flow { emit(null) } }
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
            quote?.receive
                ?.price?.takeIf { it.price.price > 0 }
                ?.currency?.let { CurrencyFormatter(currency = it).string(quote.receiveEquivalent) }
                ?: ""
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
                )
            )
        }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val uiState = session.map(::createSwapUiState)
        .stateIn(viewModelScope, SharingStarted.Eagerly, SwapUiState())

    init {
        viewModelScope.launch {
            selectedSlippageBps.value = userConfig.swapSlippageBps().firstOrNull()
        }
        matchedQuoteResults
            .onEach(::onQuoteResults)
            .launchIn(viewModelScope)
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
    ) {
        when (val action = uiState.value.action) {
            SwapActionState.Ready -> {
                if (swapDetails.value?.shouldShowPriceImpactWarning == true) {
                    onShowPriceImpactWarning()
                } else {
                    swap(onConfirm)
                }
            }
            is SwapActionState.TransferError -> swap(onConfirm)
            is SwapActionState.QuoteError -> {
                val error = action.error
                if (error is SwapError.InputAmountTooSmall) {
                    applyMinimumAmount(error)
                } else {
                    refresh()
                }
            }
            SwapActionState.None,
            SwapActionState.QuoteLoading,
            SwapActionState.TransferLoading -> Unit
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
        } catch (_: SwapNoQuoteException) {
            session.update { it.onTransferFailed(transfer, SwapError.NoQuote) }
        } catch (err: Throwable) {
            session.update { it.onTransferFailed(transfer, SwapError.Unknown(err.message ?: "")) }
        }
    }

    private fun updateBalance(id: AssetId) = viewModelScope.launch(Dispatchers.IO) {
        val currentSession = sessionRepository.session().firstOrNull() ?: return@launch
        currentSession.wallet.getAccount(id.chain) ?: return@launch
        enableAsset(currentSession.wallet.id, id)
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

    private fun applyMinimumAmount(error: SwapError.InputAmountTooSmall) {
        val asset = payAsset.value?.asset ?: return
        payValue.clearText()
        payValue.setTextAndPlaceCursorAtEnd(error.getValue(asset).toString())
    }

    private suspend fun setReceive(amount: String) = withContext(Dispatchers.Main) {
        receiveValue.clearText()
        receiveValue.setTextAndPlaceCursorAtEnd(amount)
    }

    companion object {
        val percentSuggestions = listOf(25, 50, 100)
    }
}
