package com.gemwallet.android.features.asset.viewmodels.chart.viewmodels

import android.content.Context
import android.util.Log
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.session.cases.GetCurrentCurrency
import com.gemwallet.android.data.services.gemstone.connection.ConnectionStatusObserver
import com.gemwallet.android.data.services.store.queries.PriceQuery
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.asset.viewmodels.chart.models.ChartUIModel
import com.gemwallet.android.features.asset.viewmodels.chart.models.StopTimeoutMillis
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.models.StateViewType
import com.gemwallet.android.ui.models.navigation.requireAssetId
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.ChartPeriod
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
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.gemstone.AssetPrice
import uniffi.gemstone.GemChartPhase
import uniffi.gemstone.GemChartServiceInterface
import uniffi.gemstone.GemRefreshKind
import uniffi.gemstone.GemServiceException
import javax.inject.Inject

@HiltViewModel
class ChartViewModel internal constructor(
    getCurrentCurrency: GetCurrentCurrency,
    priceQuery: PriceQuery,
    private val chartService: GemChartServiceInterface,
    private val assetId: AssetId,
    connectionStatusObserver: ConnectionStatusObserver,
    private val ioDispatcher: CoroutineDispatcher,
    private val context: Context,
) : ViewModel() {
    private val session = MutableStateFlow(chartService.newSession())

    val refreshIntervalMillis = connectionStatusObserver.refreshIntervalMillis(GemRefreshKind.CHART)
        .stateIn(viewModelScope, SharingStarted.Eagerly, 0L)

    init {
        viewModelScope.launch {
            getCurrentCurrency.getCurrency().collect { currency -> session.update { it.onCurrency(currency.toGem()) } }
        }
        viewModelScope.launch {
            session.collectLatest { current ->
                if (current.isLoading || current.isRefreshing) load()
            }
        }
    }

    private val price = priceQuery(assetId)
        .map { data ->
            data?.price?.let { price ->
                AssetPrice(
                    assetId = assetId.toIdentifier(),
                    price = price.price,
                    priceChangePercentage24h = price.priceChangePercentage24h,
                    updatedAt = price.updatedAt,
                )
            }
        }
        .distinctUntilChanged()

    private val viewState = combine(session, price) { session, price -> session.viewState(price) }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(StopTimeoutMillis), session.value.viewState(null))

    val isRefreshing: StateFlow<Boolean> = viewState.map { it.isRefreshing }
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(StopTimeoutMillis), false)

    val chartUIState = viewState.map { state ->
        ChartUIModel.State(
            period = state.period.toPrimitives(),
            chart = when (val phase = state.phase) {
                GemChartPhase.Loading -> StateViewType.Loading
                is GemChartPhase.Data -> StateViewType.Data(ChartUIModel(phase.data))
                GemChartPhase.NoData -> StateViewType.NoData
                is GemChartPhase.Failed -> StateViewType.Error(phase.error.errorText().text(context))
            },
        )
    }
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(StopTimeoutMillis), ChartUIModel.State())

    fun setPeriod(period: ChartPeriod) {
        if (period.toGem() == session.value.period) {
            return
        }
        viewModelScope.launch(ioDispatcher) {
            runCatchingCancellable { chartService.setChartPeriod(period.toGem()) }
                .onFailure { Log.e(TAG, "saving the chart period failed", it) }
        }
        session.update { it.onSelectPeriod(period.toGem()) }
    }

    fun refresh() {
        session.update { it.onRefresh() }
    }

    private suspend fun load() {
        val period = session.value.period
        try {
            val chart = withContext(ioDispatcher) { chartService.syncCharts(assetId.toIdentifier(), period) }
            session.update { it.onLoaded(chart, period) }
        } catch (e: GemServiceException) {
            session.update { it.onFailed(e, period) }
        }
    }

    @Inject
    constructor(
        getCurrentCurrency: GetCurrentCurrency,
        priceQuery: PriceQuery,
        chartService: GemChartServiceInterface,
        savedStateHandle: SavedStateHandle,
        connectionStatusObserver: ConnectionStatusObserver,
        @IoDispatcher ioDispatcher: CoroutineDispatcher,
        @ApplicationContext context: Context,
    ) : this(
        getCurrentCurrency = getCurrentCurrency,
        priceQuery = priceQuery,
        chartService = chartService,
        assetId = savedStateHandle.requireAssetId(),
        connectionStatusObserver = connectionStatusObserver,
        ioDispatcher = ioDispatcher,
        context = context,
    )
}

private const val TAG = "Chart"
