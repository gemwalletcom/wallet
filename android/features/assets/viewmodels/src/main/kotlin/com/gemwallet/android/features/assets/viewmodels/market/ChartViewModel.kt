package com.gemwallet.android.features.assets.viewmodels.market

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.assets.cases.GetWalletAssets
import com.gemwallet.android.application.connection.cases.ObserveRefreshInterval
import com.gemwallet.android.application.session.cases.GetCurrentCurrency
import com.gemwallet.android.data.services.store.queries.PriceQuery
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.ui.models.ChartUIState
import com.gemwallet.android.ui.models.navigation.requireAssetId
import com.wallet.core.primitives.AssetData
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.PriceData
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.launchIn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.shareIn
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemChartInput
import uniffi.gemstone.GemChartPhase
import uniffi.gemstone.GemChartRequest
import uniffi.gemstone.GemChartServiceInterface
import uniffi.gemstone.GemChartStyle
import uniffi.gemstone.GemChartView
import uniffi.gemstone.GemGroup
import uniffi.gemstone.GemRefreshKind
import java.time.Instant
import java.time.ZoneId
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
) : ViewModel() {
    private val session = MutableStateFlow(chartService.newSession())
    private val storedAssetInfo: AssetData? = getWalletAssets().value.firstOrNull { it.asset.id == assetId }

    val refreshIntervalMillis = observeRefreshInterval.refreshIntervalMillis(GemRefreshKind.CHART)
        .stateIn(viewModelScope, SharingStarted.Eagerly, 0L)

    private val priceData = priceQuery(assetId)
        .shareIn(viewModelScope, SharingStarted.Eagerly, replay = 1)

    init {
        viewModelScope.launch {
            getCurrentCurrency.getCurrency().collect { currency -> session.update { it.onCurrency(currency.toGem()) } }
        }
        priceData.filterNotNull()
            .map { it.properties.hasCandles }
            .distinctUntilChanged()
            .onEach { hasCandles -> session.update { it.onHasCandlesChanged(hasCandles) } }
            .launchIn(viewModelScope)
        viewModelScope.launch {
            session.map { it.request() }.distinctUntilChanged().collectLatest { request -> request?.let { load(it) } }
        }
    }

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

    val sections: StateFlow<List<GemGroup>> = view.map { it?.sections.orEmpty() }
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(StopTimeoutMillis), emptyList())

    val chartUIState: StateFlow<ChartUIState> = combine(session, view) { session, view ->
        ChartUIState(
            period = session.period.toPrimitives(),
            periods = (view?.periods ?: session.periods()).map { it.toPrimitives() },
            phase = view?.phase ?: GemChartPhase.Loading,
            styleSwitch = view?.styleSwitch,
        )
    }
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(StopTimeoutMillis), ChartUIState(period = session.value.period.toPrimitives(), periods = session.value.periods().map { it.toPrimitives() }))

    fun setPeriod(period: ChartPeriod) {
        session.update { it.onSelectPeriod(period.toGem()) }
    }

    fun selectStyle(style: GemChartStyle) {
        session.update { chartService.selectStyle(it, style) }
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
            utcOffsetSeconds = ZoneId.systemDefault().rules.getOffset(Instant.now()).totalSeconds,
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
    ) : this(
        getCurrentCurrency = getCurrentCurrency,
        priceQuery = priceQuery,
        getWalletAssets = getWalletAssets,
        chartService = chartService,
        assetId = savedStateHandle.requireAssetId(),
        observeRefreshInterval = observeRefreshInterval,
        ioDispatcher = ioDispatcher,
    )
}
