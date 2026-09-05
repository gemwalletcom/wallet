package com.gemwallet.android.features.buy.viewmodels

import android.util.Log
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.fiat.cases.GetAssetPriceUsd
import com.gemwallet.android.application.fiat.cases.GetBuyAssetInfo
import com.gemwallet.android.domains.asset.aggregates.AssetRowNaming
import com.gemwallet.android.domains.asset.aggregates.toAssetInfoDataAggregate
import com.gemwallet.android.ext.tickerFlow
import com.gemwallet.android.ext.toCurrency
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.buy.viewmodels.models.FiatSuggestion
import com.gemwallet.android.features.buy.viewmodels.models.FiatUiState
import com.gemwallet.android.features.buy.viewmodels.models.createFiatUiState
import com.gemwallet.android.features.buy.viewmodels.models.toProviderUIModel
import com.gemwallet.android.model.AssetData
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.models.navigation.requireAssetId
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.FiatProvider
import com.wallet.core.primitives.FiatQuote
import com.wallet.core.primitives.FiatQuoteType
import com.wallet.core.primitives.FiatQuoteUrl
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.FlowPreview
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.debounce
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.launchIn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.flow.mapNotNull
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.gemstone.GemFiatQuoteRequest
import uniffi.gemstone.GemFiatQuoteServiceInterface
import uniffi.gemstone.GemFiatQuotesResult
import uniffi.gemstone.GemServiceException
import java.math.BigInteger
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class, FlowPreview::class)
@HiltViewModel
class FiatViewModel @Inject constructor(
    getBuyAssetInfo: GetBuyAssetInfo,
    getAssetPriceUsd: GetAssetPriceUsd,
    private val service: GemFiatQuoteServiceInterface,
    savedStateHandle: SavedStateHandle
) : ViewModel() {

    private val currency = service.getCurrency().toCurrency()
    private val currencySymbol = java.util.Currency.getInstance(currency.name).symbol
    private val assetId: AssetId = savedStateHandle.requireAssetId(RouteArgument.AssetId)

    private val session = MutableStateFlow(
        service.newSession(
            (savedStateHandle.get<FiatQuoteType>(RouteArgument.Type.key) ?: FiatQuoteType.Buy).toGem(),
            savedStateHandle.get<Int>(RouteArgument.FiatAmount.key)?.toUInt(),
        )
    )
    private val isUrlLoading = MutableStateFlow(false)

    val type: StateFlow<FiatQuoteType> = session.map { it.quoteType.toPrimitives() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, session.value.quoteType.toPrimitives())

    val amount: StateFlow<String> = session.map { it.current().amount }
        .stateIn(viewModelScope, SharingStarted.Eagerly, session.value.current().amount)

    private val assetData: StateFlow<AssetData?> = getBuyAssetInfo(assetId)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val assetPriceUsd: StateFlow<Double?> = getAssetPriceUsd(assetId)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val assetInfoUIModel = assetData
        .mapNotNull { it }
        .map {
            val assetInfo = it.toAssetInfo()
            assetInfo.toAssetInfoDataAggregate(
                naming = AssetRowNaming.CanonicalNative,
                displayedAmount = assetInfo.balance.balanceAmount.available,
            )
        }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val showFiatTypePicker = assetData
        .filterNotNull()
        .map { it.metadata.isSellEnabled }
        .distinctUntilChanged()
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    val suggestedAmounts = type.mapLatest {
        service.config().suggestedAmounts.map {
            FiatSuggestion.SuggestionAmount("$currencySymbol$it", it.toDouble())
        } + FiatSuggestion.RandomAmount
    }.stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val uiState: StateFlow<FiatUiState> = combine(session, isUrlLoading) { session, isUrlLoading ->
        createFiatUiState(session, isUrlLoading)
    }.stateIn(viewModelScope, SharingStarted.Eagerly, createFiatUiState(session.value, false))

    val quotes: StateFlow<List<FiatQuote>> = session
        .map { it.current().quotes.map { quote -> quote.decodeJson<FiatQuote>() } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val providers = combine(assetInfoUIModel.filterNotNull(), quotes, assetPriceUsd) { asset, quotes, priceUsd ->
        quotes.map { quote -> quote.toProviderUIModel(asset.asset, currency, priceUsd) }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val selectedProvider = combine(assetInfoUIModel, session) { asset, session ->
        asset?.let { session.selectedQuote()?.decodeJson<FiatQuote>()?.toProviderUIModel(it.asset, currency) }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val ticker = tickerFlow(service.quoteRefreshIntervalMilliseconds().toLong()) {}
        .stateIn(viewModelScope, SharingStarted.Eagerly, 0L)
    private val quoteRetry = MutableStateFlow(0L)

    init {
        assetData.filterNotNull()
            .onEach { data ->
                session.update {
                    it.onBalanceChanged(BigInteger(data.balance.balance.available))
                        .onSellEnabledChanged(data.metadata.isSellEnabled)
                }
            }
            .launchIn(viewModelScope)

        combine(
            session.map { it.quoteRequest() }.distinctUntilChanged().debounce(service.quoteDebounceMilliseconds().toLong()),
            assetData.filterNotNull().map { it.asset.id }.distinctUntilChanged(),
            ticker,
            quoteRetry,
        ) { request, assetId, tick, retry -> request?.let { QuoteFetch(it, assetId, tick, retry) } }
            .distinctUntilChanged()
            .mapLatest { fetch -> fetch?.let { loadQuotes(it.request, it.assetId) } }
            .launchIn(viewModelScope)
    }

    private suspend fun loadQuotes(request: GemFiatQuoteRequest, assetId: AssetId) {
        session.update { it.onFetchStarted(request) }
        val results = try {
            GemFiatQuotesResult(request, service.quotes(request.quoteType, assetId.toIdentifier(), request.amount), null)
        } catch (err: CancellationException) {
            throw err
        } catch (err: Throwable) {
            Log.e(TAG, "fiat quotes request failed", err)
            GemFiatQuotesResult(request, emptyList(), err as? GemServiceException ?: GemServiceException.Api(err.message.orEmpty()))
        }
        session.update { it.onQuoteResults(results) }
    }

    fun updateAmount(newAmount: String) {
        session.update { it.onAmountChanged(newAmount) }
    }

    fun updateAmount(suggestion: FiatSuggestion) {
        val value = when (suggestion) {
            FiatSuggestion.RandomAmount -> service.randomAmount().toInt().toString()
            is FiatSuggestion.SuggestionAmount -> suggestion.value.toInt().toString()
        }
        updateAmount(value)
    }

    fun setProvider(provider: FiatProvider) {
        session.update { it.onProviderSelected(provider.id) }
    }

    fun setType(type: FiatQuoteType) {
        session.update { it.onTypeChanged(type.toGem()) }
    }

    fun retry() {
        quoteRetry.value += 1
    }

    fun getUrl(callback: (String?) -> Unit) {
        val quoteId = session.value.selectedQuote()?.decodeJson<FiatQuote>()?.id ?: return callback(null)
        viewModelScope.launch {
            isUrlLoading.value = true
            val url = runCatching { service.quoteUrl(assetId.toIdentifier(), quoteId).decodeJson<FiatQuoteUrl>().redirectUrl }
                .onFailure { Log.e(TAG, "fiat quote url request failed", it) }
                .getOrNull()
            isUrlLoading.value = false
            callback(url)
        }
    }

    private companion object {
        const val TAG = "FiatViewModel"
    }

    private data class QuoteFetch(
        val request: GemFiatQuoteRequest,
        val assetId: AssetId,
        val ticker: Long,
        val retry: Long,
    )
}
