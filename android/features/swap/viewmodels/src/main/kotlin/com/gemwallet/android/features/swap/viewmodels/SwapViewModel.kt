package com.gemwallet.android.features.swap.viewmodels

import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.foundation.text.input.clearText
import androidx.compose.foundation.text.input.setTextAndPlaceCursorAtEnd
import androidx.compose.runtime.snapshotFlow
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.swap.SwapRepository
import com.gemwallet.android.domains.asset.calculateFiat
import com.gemwallet.android.domains.asset.formatFiat
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.math.parseNumberOrNull
import com.gemwallet.android.features.swap.viewmodels.cases.QuoteRequester
import com.gemwallet.android.features.swap.viewmodels.models.QuoteUiState
import com.gemwallet.android.features.swap.viewmodels.models.QuoteRequestKey
import com.gemwallet.android.features.swap.viewmodels.models.QuoteState
import com.gemwallet.android.features.swap.viewmodels.models.QuoteRequestParams
import com.gemwallet.android.features.swap.viewmodels.models.QuotesState
import com.gemwallet.android.features.swap.viewmodels.models.SwapActionState
import com.gemwallet.android.features.swap.viewmodels.models.SwapError
import com.gemwallet.android.features.swap.viewmodels.models.SwapItemType
import com.gemwallet.android.features.swap.viewmodels.models.SwapUiState
import com.gemwallet.android.features.swap.viewmodels.models.TransferDataUiState
import com.gemwallet.android.features.swap.viewmodels.models.TransferQuoteSnapshot
import com.gemwallet.android.features.swap.viewmodels.models.create
import com.gemwallet.android.features.swap.viewmodels.models.createSwapUiState
import com.gemwallet.android.features.swap.viewmodels.models.formattedToAmount
import com.gemwallet.android.features.swap.viewmodels.models.getQuote
import com.gemwallet.android.features.swap.viewmodels.models.receiveEquivalent
import com.gemwallet.android.features.swap.viewmodels.models.matches
import com.gemwallet.android.features.swap.viewmodels.models.toError
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.format
import com.gemwallet.android.model.toModel
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
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.flow.launchIn
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.gemstone.SwapperProvider
import java.math.BigDecimal
import com.gemwallet.android.model.Crypto
import java.math.BigInteger
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class SwapViewModel @Inject constructor(
    private val sessionRepository: SessionRepository,
    private val assetsRepository: AssetsRepository,
    private val swapRepository: SwapRepository,
    quoteRequester: QuoteRequester,
    private val savedStateHandle: SavedStateHandle
) : ViewModel() {

    private val quoteUiState = MutableStateFlow<QuoteUiState>(QuoteUiState.NoInput)
    private val transferDataUiState = MutableStateFlow<TransferDataUiState>(TransferDataUiState.Idle)
    private val transferQuoteSnapshot = MutableStateFlow<TransferQuoteSnapshot?>(null)

    val payValue: TextFieldState = TextFieldState()
    val receiveValue: TextFieldState = TextFieldState()

    private val payValueFlow = snapshotFlow { payValue.text }
        .map { it.toString() }
        .map { it.parseNumberOrNull() ?: BigDecimal.ZERO }
        .stateIn(viewModelScope, SharingStarted.Eagerly, BigDecimal.ZERO)

    val selectedProvider = MutableStateFlow<SwapperProvider?>(null)

    private val refreshRequests = MutableSharedFlow<Unit>(extraBufferCapacity = 1)
    private val refreshEnabled = MutableStateFlow(false)
    private val pauseQuoteRefreshUntilNextStart = MutableStateFlow(false)
    private val quoteRefreshEnabled = combine(refreshEnabled, transferDataUiState, pauseQuoteRefreshUntilNextStart) { isEnabled, transferState, isPaused ->
            isEnabled && !isPaused && transferState !is TransferDataUiState.Loading
        }

    val payAsset = savedStateHandle.getStateFlow<String?>("from", null)
        .map { it?.toAssetId() }
        .onEach { id -> id?.let { updateBalance(it) } }
        .flatMapLatest { assetId -> assetId?.let { assetsRepository.getAssetInfo(it) } ?: flow { emit(null) } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val receiveAsset = savedStateHandle.getStateFlow<String?>("to", null)
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

    private val quoteRequestParams = combine(payValueFlow, payAsset, receiveAsset) { value, fromAsset, toAsset ->
            QuoteRequestParams.create(value, fromAsset, toAsset)
        }
        .onEach(::onQuoteRequestParamsChanged)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val quoteResults = quoteRequester.requestQuotes(
        requestParams = quoteRequestParams,
        refreshRequests = refreshRequests,
        refreshEnabled = quoteRefreshEnabled,
        onFetchStarted = ::onQuoteFetchStarted,
    )
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val matchedQuoteResults = combine(quoteRequestParams, quoteResults) { params, results ->
            results?.takeIf { it.matches(params) }
        }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val liveQuotes = matchedQuoteResults
        .mapLatest { results ->
            results?.takeIf { it.err == null && it.items.isNotEmpty() }
        }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val activeTransferSnapshot = combine(transferQuoteSnapshot, transferDataUiState) { frozen, transferState ->
            when (transferState) {
                TransferDataUiState.Idle -> null
                is TransferDataUiState.Error,
                is TransferDataUiState.Loading -> frozen
            }
        }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val displayedQuotes = combine(liveQuotes, activeTransferSnapshot) { live, frozen ->
            frozen?.quotes ?: live
        }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val displayedProvider = combine(selectedProvider, activeTransferSnapshot) { provider, frozen ->
            frozen?.selectedProvider ?: provider
        }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val providers = displayedQuotes.mapLatest { quotes ->
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

    val quote = combine(displayedQuotes, displayedProvider) { quotes, provider ->
            quotes?.getQuote(provider)?.let { QuoteState(it, quotes.pay, quotes.receive) }
        }
        .onEach { state -> setReceive(state?.formattedToAmount ?: "") }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val toEquivalentFormatted = quote.mapLatest { quote ->
            quote?.receive
                ?.price?.takeIf { it.price.price > 0 }
                ?.currency?.format(quote.receiveEquivalent, dynamicPlace = true)
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
                    etaInSeconds = quote.quote.etaInSeconds,
                    isProviderSelectable = providers.size > 1,
                )
            )
        }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val uiState = combine(quoteUiState, transferDataUiState, quote) { quoteState, transferState, displayedQuote ->
            createSwapUiState(
                quoteState = quoteState,
                transferState = transferState,
                displayedQuote = displayedQuote,
            )
        }
        .stateIn(viewModelScope, SharingStarted.Eagerly, SwapUiState())

    init {
        matchedQuoteResults
            .onEach(::onQuoteResults)
            .launchIn(viewModelScope)
    }

    fun onSelect(type: SwapItemType, assetId: AssetId) {
        clearTransferQuoteState()
        when (type) {
            SwapItemType.Pay -> {
                if (receiveAsset.value?.id() == assetId) {
                    savedStateHandle["to"] = null
                }
                savedStateHandle["from"] = assetId.toIdentifier()
                payValue.clearText()
            }
            SwapItemType.Receive -> {
                if (payAsset.value?.id() == assetId) {
                    savedStateHandle["from"] = null
                }
                savedStateHandle["to"] = assetId.toIdentifier()
            }
        }
    }

    fun onSelectPayPercent(percent: Int) {
        val asset = payAsset.value ?: return
        val available = asset.balance.balance.available.toBigIntegerOrNull() ?: return
        val scaled = available.multiply(percent.toBigInteger()).divide(100.toBigInteger())
        val formatted = Crypto(scaled)
            .value(asset.asset.decimals)
            .stripTrailingZeros()
            .toPlainString()
        clearTransferQuoteState()
        payValue.clearText()
        payValue.setTextAndPlaceCursorAtEnd(formatted)
    }

    fun switchSwap() = viewModelScope.launch {
        clearTransferQuoteState()
        val payAssetId = payAsset.value?.id()?.toIdentifier()
        val receiveAssetId = receiveAsset.value?.id()?.toIdentifier()
        savedStateHandle["from"] = receiveAssetId
        savedStateHandle["to"] = payAssetId
        payValue.clearText()
    }

    fun setProvider(provider: SwapperProvider) {
        clearTransferQuoteState()
        this.selectedProvider.update { provider }
    }

    fun refresh() {
        val params = quoteRequestParams.value ?: return
        clearTransferQuoteState()
        quoteUiState.value = QuoteUiState.Loading(params.key)
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
        if (isEnabled && !refreshEnabled.value && pauseQuoteRefreshUntilNextStart.value) {
            pauseQuoteRefreshUntilNextStart.value = false
        }
        refreshEnabled.value = isEnabled
    }

    fun swap(onConfirm: (ConfirmParams) -> Unit) = viewModelScope.launch(Dispatchers.IO) {
        if (transferDataUiState.value is TransferDataUiState.Loading) return@launch

        val snapshot = currentQuoteSnapshot() ?: return@launch
        transferQuoteSnapshot.value = snapshot
        transferDataUiState.value = TransferDataUiState.Loading(
            quoteKey = snapshot.requestKey,
            providerId = snapshot.providerId,
        )

        try {
            val params = swap(snapshot) ?: run {
                if (transferDataUiState.value.matches(snapshot)) {
                    clearTransferQuoteState()
                }
                return@launch
            }
            if (!transferDataUiState.value.matches(snapshot)) {
                return@launch
            }
            withContext(Dispatchers.Main) {
                onConfirm(params)
            }
            if (transferDataUiState.value.matches(snapshot)) {
                pauseQuoteRefreshUntilNextStart.value = true
                clearTransferQuoteState(resumeQuoteRefresh = false)
            }
        } catch (err: SwapError) {
            if (transferDataUiState.value.matches(snapshot)) {
                transferDataUiState.value = TransferDataUiState.Error(
                    quoteKey = snapshot.requestKey,
                    providerId = snapshot.providerId,
                    error = err,
                )
            }
        } catch (err: Throwable) {
            if (transferDataUiState.value.matches(snapshot)) {
                transferDataUiState.value = TransferDataUiState.Error(
                    quoteKey = snapshot.requestKey,
                    providerId = snapshot.providerId,
                    error = SwapError.Unknown(err.message ?: ""),
                )
            }
        }
    }

    private suspend fun swap(snapshot: TransferQuoteSnapshot): ConfirmParams? {
        val quote = snapshot.quote
        val fromAmount = BigInteger(quote.quote.fromValue)
        val wallet = sessionRepository.session().firstOrNull()?.wallet ?: return null

        val swapData = try {
            swapRepository.getQuoteData(quote.quote, wallet)
        } catch (_: Throwable) {
            throw SwapError.NoQuote
        }

        return ConfirmParams.SwapParams(
            from = quote.pay.owner ?: throw SwapError.NoQuote,
            fromAsset = quote.pay.asset,
            toAsset = quote.receive.asset,
            fromAmount = fromAmount,
            toAmount = BigInteger(quote.quote.toValue),
            swapData = swapData.data,
            providerId = quote.quote.data.provider.id,
            protocol = quote.quote.data.provider.protocol,
            providerName = quote.quote.data.provider.name,
            protocolId = quote.quote.data.provider.protocolId,
            toAddress = swapData.to,
            value = swapData.value,
            approval = swapData.approval?.toModel(),
            gasLimit = swapData.gasLimit?.toBigIntegerOrNull(),
            useMaxAmount = quote.quote.request.options.useMaxAmount,
            etaInSeconds = quote.quote.etaInSeconds,
            slippageBps = quote.quote.data.slippageBps,
            memo = swapData.memo,
            dataType = swapData.dataType,
        )
    }

    private fun updateBalance(id: AssetId) = viewModelScope.launch(Dispatchers.IO) {
        val session = sessionRepository.session().firstOrNull() ?: return@launch
        val account = session.wallet.getAccount(id.chain) ?: return@launch
        assetsRepository.switchVisibility(session.wallet.id, account, id, true)
    }

    private fun onQuoteRequestParamsChanged(params: QuoteRequestParams?) {
        if (params == null) {
            selectedProvider.update { null }
            clearTransferQuoteState()
            quoteUiState.value = QuoteUiState.NoInput
            return
        }

        clearTransferQuoteState()
        quoteUiState.value = QuoteUiState.Loading(params.key)
    }

    private fun onQuoteFetchStarted(requestKey: QuoteRequestKey) {
        if (transferDataUiState.value is TransferDataUiState.Loading || pauseQuoteRefreshUntilNextStart.value) {
            return
        }
        quoteUiState.value = QuoteUiState.Loading(requestKey)
    }

    private fun onQuoteResults(results: QuotesState?) {
        if (results == null || transferDataUiState.value is TransferDataUiState.Loading || pauseQuoteRefreshUntilNextStart.value) {
            return
        }

        quoteUiState.value = when {
            results.err != null -> QuoteUiState.Error(results.requestKey, SwapError.toError(results.err))
            results.items.isEmpty() -> QuoteUiState.Error(results.requestKey, SwapError.NoQuote)
            else -> QuoteUiState.Ready(results)
        }
    }

    private fun currentQuoteSnapshot(): TransferQuoteSnapshot? {
        transferQuoteSnapshot.value
            ?.takeIf { transferDataUiState.value != TransferDataUiState.Idle }
            ?.let { return it }

        val quotes = (quoteUiState.value as? QuoteUiState.Ready)?.quotes ?: return null
        return TransferQuoteSnapshot.create(
            quotes = quotes,
            selectedProvider = selectedProvider.value,
        )
    }

    private fun clearTransferQuoteState(resumeQuoteRefresh: Boolean = true) {
        if (resumeQuoteRefresh) {
            pauseQuoteRefreshUntilNextStart.value = false
        }
        transferDataUiState.value = TransferDataUiState.Idle
        transferQuoteSnapshot.value = null
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
}
