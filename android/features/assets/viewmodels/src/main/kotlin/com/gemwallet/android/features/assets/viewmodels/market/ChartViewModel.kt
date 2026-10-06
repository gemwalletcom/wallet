package com.gemwallet.android.features.assets.viewmodels.market

import android.content.Context
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.assets.cases.GetWalletAssets
import com.gemwallet.android.application.connection.cases.ObserveRefreshInterval
import com.gemwallet.android.application.session.cases.GetCurrentCurrency
import com.gemwallet.android.data.services.store.queries.PriceQuery
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.models.ChartUIState
import com.gemwallet.android.ui.models.StateViewType
import com.gemwallet.android.ui.models.navigation.requireAssetId
import com.wallet.core.primitives.AssetData
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.PriceData
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.shareIn
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemChartInput
import uniffi.gemstone.GemChartPhase
import uniffi.gemstone.GemChartRequest
import uniffi.gemstone.GemChartServiceInterface
import uniffi.gemstone.GemChartView
import uniffi.gemstone.GemListSection
import uniffi.gemstone.GemRefreshKind
import javax.inject.Inject

private const val StopTimeoutMillis = 5_000L

@HiltViewModel
class ChartViewModel internal constructor(
    getCurrentCurrency: GetCurrentCurrency,
    priceQuery: PriceQuery,
    getWalletAssets: GetWalletAssets,
    private val chartService: GemChartServiceInterface,
    val assetId: AssetId,
    observeRefreshInterval: ObserveRefreshInterval,
    private val ioDispatcher: CoroutineDispatcher,
    private val context: Context,
) : ViewModel() {
    private val session = MutableStateFlow(chartService.newSession())
    private val storedAssetInfo: AssetData? = getWalletAssets().value.firstOrNull { it.asset.id == assetId }

    val refreshIntervalMillis = observeRefreshInterval.refreshIntervalMillis(GemRefreshKind.CHART)
        .stateIn(viewModelScope, SharingStarted.Eagerly, 0L)

    init {
        viewModelScope.launch {
            getCurrentCurrency.getCurrency().collect { currency -> session.update { it.onCurrency(currency.toGem()) } }
        }
        viewModelScope.launch {
            session.map { it.request() }.distinctUntilChanged().collectLatest { request -> request?.let { load(it) } }
        }
    }

    private val priceData = priceQuery(assetId)
        .shareIn(viewModelScope, SharingStarted.Eagerly, replay = 1)

    val title = priceData
        .map { it?.asset?.name.orEmpty() }
        .distinctUntilChanged()
        .stateIn(viewModelScope, SharingStarted.Eagerly, storedAssetInfo?.asset?.name.orEmpty())

    private val view: StateFlow<GemChartView?> = combine(session, priceData) { session, data ->
        chartInput(data)?.let { chartService.viewState(session, it) }
    }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(StopTimeoutMillis), null)

    val isRefreshing: StateFlow<Boolean> = session.map { it.isRefreshing }
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(StopTimeoutMillis), false)

    val sections: StateFlow<List<GemListSection>> = view.map { it?.sections.orEmpty() }
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(StopTimeoutMillis), emptyList())

    val chartUIState: StateFlow<ChartUIState> = combine(session, view) { session, view ->
        ChartUIState(
            period = session.period.toPrimitives(),
            chart = when (val phase = view?.phase) {
                null, GemChartPhase.Loading -> StateViewType.Loading
                is GemChartPhase.Data -> StateViewType.Data(phase.data)
                GemChartPhase.NoData -> StateViewType.NoData
                is GemChartPhase.Failed -> StateViewType.Error(phase.error.errorText().text(context))
            },
        )
    }
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(StopTimeoutMillis), ChartUIState(period = session.value.period.toPrimitives()))

    fun setPeriod(period: ChartPeriod) {
        session.update { it.onSelectPeriod(period.toGem()) }
    }

    fun refresh() {
        session.update { it.onRefresh() }
    }

    fun onZoom(magnification: Float, anchor: Float) {
        session.update { it.onZoom(magnification.toDouble(), anchor.toDouble()) }
    }

    fun onPan(fraction: Float) {
        session.update { it.onPan(fraction.toDouble()) }
    }

    private fun chartInput(data: PriceData?): GemChartInput? {
        val asset = data?.asset ?: storedAssetInfo?.asset ?: return null
        return GemChartInput(
            asset = asset.toGem(),
            price = data?.price?.toGem(),
            market = data?.market?.toGem(),
            priceAlerts = data?.priceAlerts.orEmpty().map { it.toGem() },
            links = data?.links.orEmpty().map { it.toGem() },
        )
    }

    private suspend fun load(request: GemChartRequest) {
        val result = withContext(ioDispatcher) { chartService.load(assetId.toIdentifier(), request) }
        session.update { it.onResult(result) }
    }

    @Inject
    constructor(
        getCurrentCurrency: GetCurrentCurrency,
        priceQuery: PriceQuery,
        getWalletAssets: GetWalletAssets,
        chartService: GemChartServiceInterface,
        savedStateHandle: SavedStateHandle,
        observeRefreshInterval: ObserveRefreshInterval,
        @IoDispatcher ioDispatcher: CoroutineDispatcher,
        @ApplicationContext context: Context,
    ) : this(
        getCurrentCurrency = getCurrentCurrency,
        priceQuery = priceQuery,
        getWalletAssets = getWalletAssets,
        chartService = chartService,
        assetId = savedStateHandle.requireAssetId(),
        observeRefreshInterval = observeRefreshInterval,
        ioDispatcher = ioDispatcher,
        context = context,
    )
}
