package com.gemwallet.android.features.buy.viewmodels

import android.content.Context
import android.util.Log
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.store.queries.AssetQueryOptional
import com.gemwallet.android.data.services.store.queries.PriceUsdQuery
import com.gemwallet.android.domains.asset.aggregates.toAssetInfoDataAggregate
import com.gemwallet.android.ext.GemConstants
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.tickerFlow
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.buy.viewmodels.models.FiatUiState
import com.gemwallet.android.features.buy.viewmodels.models.createFiatUiState
import com.gemwallet.android.features.buy.viewmodels.models.toProviderUIModel
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.localization.quotesMessage
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.models.navigation.requireAssetId
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.FiatProviderName
import com.wallet.core.primitives.FiatQuoteType
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.FlowPreview
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.debounce
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.launchIn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.flow.mapNotNull
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemAssetBalanceScope
import uniffi.gemstone.GemFiatQuoteRequest
import uniffi.gemstone.GemFiatQuoteServiceInterface
import uniffi.gemstone.GemFiatQuotesResult
import uniffi.gemstone.GemFiatSuggestedAmount
import uniffi.gemstone.GemFiatViewState
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemSelectAssetType
import uniffi.gemstone.GemServiceException
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class, FlowPreview::class)
@HiltViewModel
class FiatViewModel @Inject constructor(
    getSession: GetSession,
    assetQuery: AssetQueryOptional,
    priceUsdQuery: PriceUsdQuery,
    private val service: GemFiatQuoteServiceInterface,
    @ApplicationContext private val context: Context,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    savedStateHandle: SavedStateHandle,
) : ViewModel() {

    private val currency = GemConstants.fiatQuoteCurrency
    private val assetId: AssetId = savedStateHandle.requireAssetId(RouteArgument.AssetId)

    private val session = MutableStateFlow(
        service.newSession(
            (savedStateHandle.get<FiatQuoteType>(RouteArgument.Type.key) ?: FiatQuoteType.Buy).toGem(),
            savedStateHandle.get<Int>(RouteArgument.FiatAmount.key)?.toUInt(),
        ),
    )
    private val isUrlLoading = MutableStateFlow(false)

    val type: StateFlow<FiatQuoteType> = session.map { it.quoteType.toPrimitives() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, session.value.quoteType.toPrimitives())

    private val assetInfo: StateFlow<AssetInfo?> = combine(
        getSession(),
        getSession().filterNotNull().map { it.wallet.id.id }.distinctUntilChanged().flatMapLatest { walletId -> assetQuery(walletId, assetId) },
    ) { walletSession, info -> info?.takeIf { walletSession?.wallet?.getAccount(assetId) != null } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val assetPriceUsd: StateFlow<Double?> = priceUsdQuery(assetId)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val viewState = combine(session, isUrlLoading, assetPriceUsd, assetInfo) { session, isUrlLoading, priceUsd, assetInfo ->
        session.viewState(priceUsd, isUrlLoading, assetInfo?.metadata?.isSellEnabled == true)
    }.stateIn(viewModelScope, SharingStarted.Eagerly, session.value.viewState(null, false, false))

    val amount: StateFlow<String> = viewState.map { it.amount }
        .stateIn(viewModelScope, SharingStarted.Eagerly, viewState.value.amount)

    val assetInfoUIModel = assetInfo
        .mapNotNull { it }
        .map {
            it.toAssetInfoDataAggregate(
                style = GemSelectAssetType.Buy.flow().rowStyle,
                scope = GemAssetBalanceScope.AVAILABLE,
            )
        }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val showsTypePicker: StateFlow<Boolean> = viewState.map { it.showsTypePicker }
        .stateIn(viewModelScope, SharingStarted.Eagerly, viewState.value.showsTypePicker)

    val suggestedAmounts: List<GemFiatSuggestedAmount> = service.suggestedAmounts()

    val uiState: StateFlow<FiatUiState> = combine(viewState, assetInfoUIModel) { state, asset ->
        state.toUiState()
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, viewState.value.toUiState())

    val providers = combine(assetInfoUIModel.filterNotNull(), viewState) { _, state ->
        state.quoteRows.map { row -> row.toProviderUIModel() }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val selectedProvider = combine(assetInfoUIModel, viewState) { asset, state ->
        asset?.let { state.selectedQuoteRow?.toProviderUIModel() }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val providerListItem: StateFlow<ListItemModel?> = selectedProvider.map { provider ->
        provider?.let {
            ListItemModel(
                title = context.getString(R.string.common_provider),
                subtitle = it.providerName,
            )
        }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val rateRow: StateFlow<GemListRow?> = viewState.map { it.rateRow }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val refreshEnabled = MutableStateFlow(false)
    private val ticker = combine(refreshEnabled, session) { isEnabled, quoteSession -> quoteSession.refreshesQuotes(isEnabled) }
        .distinctUntilChanged()
        .flatMapLatest { refreshes -> if (refreshes) tickerFlow(GemConstants.fiatQuoteRefreshInterval.inWholeMilliseconds) {} else emptyFlow() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, 0L)
    private val quoteRetry = MutableStateFlow(0L)

    init {
        assetInfo.filterNotNull()
            .onEach { data ->
                session.update {
                    it.onBalanceChanged(data.balance.balance.available)
                        .onSellEnabledChanged(data.metadata.isSellEnabled)
                }
            }
            .launchIn(viewModelScope)

        combine(
            session.map { it.quoteRequest() }.distinctUntilChanged().debounce(GemConstants.fiatQuoteDebounce),
            assetInfo.filterNotNull().map { it.asset.id }.distinctUntilChanged(),
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
            GemFiatQuotesResult(request, withContext(ioDispatcher) { service.quotes(request.quoteType, assetId.toIdentifier(), request.amount) }, null)
        } catch (err: GemServiceException) {
            Log.e(TAG, "fiat quotes request failed", err)
            GemFiatQuotesResult(request, emptyList(), err)
        }
        session.update { it.onQuoteResults(results) }
    }

    fun updateAmount(newAmount: String) {
        session.update { it.onAmountChanged(newAmount) }
    }

    fun selectAmount(suggestion: GemFiatSuggestedAmount) {
        updateAmount(suggestion.amount.toString())
    }

    fun selectRandomAmount() {
        updateAmount(service.randomAmount().toString())
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

    fun setRefreshEnabled(isEnabled: Boolean) {
        refreshEnabled.value = isEnabled
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

    private fun GemFiatViewState.toUiState(): FiatUiState = createFiatUiState(
        state = this,
        amountError = amountError?.string(context),
        quotesMessage = quotesMessage(context),
    )

    private companion object {
        const val TAG = "FiatViewModel"
    }

    private data class QuoteFetch(val request: GemFiatQuoteRequest, val assetId: AssetId, val ticker: Long, val retry: Long)
}
