package com.gemwallet.android.features.buy.viewmodels

import android.util.Log
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.fiat.cases.GetAssetPriceUsd
import com.gemwallet.android.application.fiat.cases.GetBuyAssetInfo
import uniffi.gemstone.GemAssetRowTitle
import com.gemwallet.android.domains.asset.aggregates.toAssetInfoDataAggregate
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.tickerFlow
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.buy.viewmodels.models.FiatSuggestion
import com.gemwallet.android.features.buy.viewmodels.models.FiatUiState
import com.gemwallet.android.features.buy.viewmodels.models.createFiatUiState
import com.gemwallet.android.features.buy.viewmodels.models.toProviderUIModel
import com.gemwallet.android.model.AssetData
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.models.navigation.requireAssetId
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.FiatProviderName
import com.wallet.core.primitives.FiatQuoteType
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
import uniffi.gemstone.GemFiatQuoteRequest
import uniffi.gemstone.GemFiatQuoteServiceInterface
import uniffi.gemstone.GemFiatQuotesResult
import uniffi.gemstone.GemServiceException
import javax.inject.Inject
import uniffi.gemstone.GemFiatViewState
import uniffi.gemstone.GemFiatQuotePhase
import uniffi.gemstone.GemFiatAmountCheck
import dagger.hilt.android.qualifiers.ApplicationContext
import com.gemwallet.android.ui.R
import android.content.Context
import com.gemwallet.android.model.text

@OptIn(ExperimentalCoroutinesApi::class, FlowPreview::class)
@HiltViewModel
class FiatViewModel @Inject constructor(
    getBuyAssetInfo: GetBuyAssetInfo,
    getAssetPriceUsd: GetAssetPriceUsd,
    private val service: GemFiatQuoteServiceInterface,
    @ApplicationContext private val context: Context,
    savedStateHandle: SavedStateHandle
) : ViewModel() {

    private val currency = service.getCurrency().toPrimitives()
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

    private val assetData: StateFlow<AssetData?> = getBuyAssetInfo(assetId)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val assetPriceUsd: StateFlow<Double?> = getAssetPriceUsd(assetId)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val viewState = combine(session, isUrlLoading, assetPriceUsd) { session, isUrlLoading, priceUsd ->
        session.viewState(priceUsd, isUrlLoading)
    }.stateIn(viewModelScope, SharingStarted.Eagerly, session.value.viewState(null, false))

    val amount: StateFlow<String> = viewState.map { it.amount }
        .stateIn(viewModelScope, SharingStarted.Eagerly, viewState.value.amount)

    val assetInfoUIModel = assetData
        .mapNotNull { it }
        .map {
            val assetInfo = it.toAssetInfo()
            assetInfo.toAssetInfoDataAggregate(
                naming = GemAssetRowTitle.CANONICAL_ASSET,
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
        service.suggestedAmounts().map {
            FiatSuggestion.SuggestionAmount("$currencySymbol$it", it.toDouble())
        } + FiatSuggestion.RandomAmount
    }.stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val uiState: StateFlow<FiatUiState> = combine(viewState, assetInfoUIModel) { state, asset ->
        createFiatUiState(state, errorText(state, asset?.asset?.name.orEmpty(), asset?.asset?.symbol.orEmpty()))
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, createFiatUiState(viewState.value, null))

    private fun errorText(state: GemFiatViewState, assetName: String, assetSymbol: String): String? = when (val phase = state.phase) {
        is GemFiatQuotePhase.Invalid -> amountCheckText(phase.check, assetName, assetSymbol)
        GemFiatQuotePhase.InvalidInput -> context.getString(R.string.errors_invalid_amount)
        GemFiatQuotePhase.NoInput -> context.getString(
            R.string.input_enter_amount_to,
            context.getString(if (state.quoteType.toPrimitives() == FiatQuoteType.Buy) R.string.buy_title else R.string.sell_title, ""),
        )
        GemFiatQuotePhase.NoQuotes -> context.getString(R.string.buy_no_results)
        is GemFiatQuotePhase.Failed -> context.getString(R.string.errors_unknown_try_again)
        is GemFiatQuotePhase.Loading -> null
        GemFiatQuotePhase.Ready -> amountCheckText(state.amountCheck, assetName, assetSymbol)
    }

    private fun amountCheckText(check: GemFiatAmountCheck, assetName: String, assetSymbol: String): String? = when (check) {
        is GemFiatAmountCheck.BelowMinimum -> context.getString(R.string.transfer_minimum_amount, check.minimum.text())
        is GemFiatAmountCheck.AboveMaximum -> context.getString(R.string.transfer_maximum_amount, check.maximum.text())
        is GemFiatAmountCheck.InsufficientBalance -> context.getString(R.string.transfer_insufficient_balance, "$assetName ($assetSymbol)")
        GemFiatAmountCheck.Valid -> null
    }

    val providers = combine(assetInfoUIModel.filterNotNull(), viewState) { asset, state ->
        state.quoteRows.map { row -> row.toProviderUIModel(asset.asset, currency) }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val selectedProvider = combine(assetInfoUIModel, viewState) { asset, state ->
        asset?.let { state.selectedQuoteRow?.toProviderUIModel(it.asset, currency) }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val ticker = tickerFlow(service.quoteRefreshIntervalMilliseconds().toLong()) {}
        .stateIn(viewModelScope, SharingStarted.Eagerly, 0L)
    private val quoteRetry = MutableStateFlow(0L)

    init {
        assetData.filterNotNull()
            .onEach { data ->
                session.update {
                    it.onBalanceChanged(data.balance.balance.available)
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

    fun setProvider(provider: FiatProviderName) {
        session.update { it.onProviderSelected(provider.toGem()) }
    }

    fun setType(type: FiatQuoteType) {
        session.update { it.onTypeChanged(type.toGem()) }
    }

    fun retry() {
        quoteRetry.value += 1
    }

    suspend fun quoteUrl(): Result<String> {
        val quoteId = requireNotNull(viewState.value.selectedQuoteRow).quoteId
        isUrlLoading.value = true
        try {
            return runCatchingCancellable { service.quoteUrl(assetId.toIdentifier(), quoteId).redirectUrl }
                .onFailure { Log.e(TAG, "fiat quote url request failed", it) }
        } finally {
            isUrlLoading.value = false
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
