package com.gemwallet.android.features.swap.viewmodels

import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.foundation.text.input.clearText
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
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.format
import com.gemwallet.android.model.toModel
import com.gemwallet.android.features.swap.viewmodels.cases.QuoteRequester
import com.gemwallet.android.features.swap.viewmodels.models.QuoteState
import com.gemwallet.android.features.swap.viewmodels.models.SwapError
import com.gemwallet.android.features.swap.viewmodels.models.SwapItemType
import com.gemwallet.android.features.swap.viewmodels.models.SwapState
import com.gemwallet.android.features.swap.viewmodels.models.create
import com.gemwallet.android.features.swap.viewmodels.models.formattedToAmount
import com.gemwallet.android.features.swap.viewmodels.models.getQuote
import com.gemwallet.android.features.swap.viewmodels.models.QuoteRequestParams
import com.gemwallet.android.features.swap.viewmodels.models.receiveEquivalent
import com.gemwallet.android.features.swap.viewmodels.models.validate
import com.gemwallet.android.features.swap.viewmodels.models.matches
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
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.gemstone.SwapperProvider
import java.math.BigDecimal
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

    private val swapScreenState = MutableStateFlow<SwapState>(SwapState.None)

    val payValue: TextFieldState = TextFieldState()
    val receiveValue: TextFieldState = TextFieldState()

    private val payValueFlow = snapshotFlow { payValue.text }
        .map { it.toString() }
        .map { it.parseNumberOrNull() ?: BigDecimal.ZERO }
        .stateIn(viewModelScope, SharingStarted.Eagerly, BigDecimal.ZERO)

    val selectedProvider = MutableStateFlow<SwapperProvider?>(null)

    private val refreshRequests = MutableSharedFlow<Unit>(extraBufferCapacity = 1)
    private val refreshEnabled = MutableStateFlow(false)

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
        .flowOn(Dispatchers.Default)
        .stateIn(viewModelScope, SharingStarted.Eagerly, "")

    private val quoteRequestParams = combine(payValueFlow, payAsset, receiveAsset) { value, fromAsset, toAsset ->
            QuoteRequestParams.create(value, fromAsset, toAsset)
        }
        .onEach { params ->
            if (params == null) {
                selectedProvider.update { null }
                swapScreenState.update { SwapState.None }
            } else {
                swapScreenState.update { SwapState.GetQuote }
            }
        }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val quoteResults = quoteRequester.requestQuotes(
        requestParams = quoteRequestParams,
        refreshRequests = refreshRequests,
        refreshEnabled = refreshEnabled,
        onError = { err ->
            swapScreenState.update { SwapState.Error.create(err) }
        },
    )
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val quotes = combine(quoteRequestParams, quoteResults) { params, results ->
            results?.takeIf { it.matches(params) }
        }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val providers = quotes.mapLatest { quotes ->
            val quoteState = quotes ?: return@mapLatest emptyList()
            quoteState.items.map { item ->
                SwapProviderUIModelFactory.create(
                    provider = item.data.provider,
                    receiveAsset = quoteState.receive,
                    toValue = item.toValue,
                )
            }
        }
        .flowOn(Dispatchers.Default)
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val quote = combine(quotes, selectedProvider) { quotes, provider ->
            quotes?.getQuote(provider)?.let { QuoteState(it, quotes.pay, quotes.receive) }
        }
        .onEach { state -> setReceive(state?.formattedToAmount ?: "") }
        .onEach { state -> state?.let { s -> swapScreenState.update { s.validate() } } }
        .flowOn(Dispatchers.Default)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val currentProvider = quote.mapLatest { it?.quote?.data?.provider?.id }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val toEquivalentFormatted = quote.mapLatest { quote ->
            quote?.receive
                ?.price?.takeIf { it.price.price > 0 }
                ?.currency?.format(quote.receiveEquivalent, dynamicPlace = true)
                ?: ""
        }
        .flowOn(Dispatchers.Default)
        .stateIn(viewModelScope, SharingStarted.Eagerly, "")

    val swapDetails = combine(quote, providers, swapScreenState) { quote, providers, state ->
            if (quote == null || state == SwapState.GetQuote) {
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
        .flowOn(Dispatchers.Default)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val uiSwapScreenState = swapScreenState
        .stateIn(viewModelScope, SharingStarted.Eagerly, SwapState.None)

    fun onSelect(type: SwapItemType, assetId: AssetId) {
        when (type) {
            SwapItemType.Pay -> {
                if (receiveAsset.value?.id() == assetId) {
                    savedStateHandle["to"] = null
                }
                savedStateHandle["from"] = assetId.toIdentifier()
            }
            SwapItemType.Receive -> {
                if (payAsset.value?.id() == assetId) {
                    savedStateHandle["from"] = null
                }
                savedStateHandle["to"] = assetId.toIdentifier()
            }
        }
    }

    fun switchSwap() = viewModelScope.launch {
        val payAssetId = payAsset.value?.id()?.toIdentifier()
        val receiveAssetId = receiveAsset.value?.id()?.toIdentifier()
        savedStateHandle["from"] = receiveAssetId
        savedStateHandle["to"] = payAssetId
        payValue.clearText()
        swapScreenState.update { SwapState.None }
    }

    fun setProvider(provider: SwapperProvider) {
        this.selectedProvider.update { provider }
    }

    fun refresh() {
        swapScreenState.update { SwapState.GetQuote }
        refreshRequests.tryEmit(Unit)
    }

    fun setRefreshEnabled(isEnabled: Boolean) {
        refreshEnabled.value = isEnabled
    }

    fun swap(onConfirm: (ConfirmParams) -> Unit) = viewModelScope.launch(Dispatchers.IO) {
        if (swapScreenState.value == SwapState.Swapping) return@launch

        swapScreenState.update { SwapState.Swapping }

        try {
            val params = swap() ?: return@launch
            swapScreenState.update { SwapState.Ready }
            withContext(Dispatchers.Main) {
                onConfirm(params)
            }
        } catch (err: SwapError) {
            swapScreenState.update { SwapState.Error(err) }
        } catch (err: Throwable) {
            swapScreenState.update { SwapState.Error(SwapError.Unknown(err.message ?: "")) }
        }
    }

    private suspend fun swap(): ConfirmParams? {
        val fromAmount = payValue.text.toString().parseNumberOrNull() ?: throw SwapError.IncorrectInput
        val quote = quote.value ?: throw SwapError.NoQuote
        val wallet = sessionRepository.session().firstOrNull()?.wallet ?: return null

        val swapData = try {
            swapRepository.getQuoteData(quote.quote, wallet)
        } catch (_: Throwable) {
            throw SwapError.NoQuote
        }

        return ConfirmParams.SwapParams(
            from = quote.pay.owner!!,
            fromAsset = quote.pay.asset,
            toAsset = quote.receive.asset,
            fromAmount = Crypto(fromAmount, quote.pay.asset.decimals).atomicValue,
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
            useMaxAmount = quote.quote.request.options.useMaxAmount/* BigInteger(quote.pay.balance.balance.available) == Crypto(fromAmount, quote.pay.asset.decimals).atomicValue*/,
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

    private suspend fun setReceive(amount: String) = withContext(Dispatchers.Main) {
        receiveValue.edit { replace(0, length, amount) }
    }
}
