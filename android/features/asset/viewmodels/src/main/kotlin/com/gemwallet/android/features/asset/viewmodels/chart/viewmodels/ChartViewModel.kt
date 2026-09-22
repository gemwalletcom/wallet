package com.gemwallet.android.features.asset.viewmodels.chart.viewmodels

import android.util.Log
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.assets.cases.GetAssetTokenInfo
import com.gemwallet.android.application.session.cases.GetCurrentCurrency
import com.gemwallet.android.data.services.gemstone.connection.ConnectionStatusObserver
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.asset.viewmodels.chart.models.ChartUIModel
import com.gemwallet.android.features.asset.viewmodels.chart.models.StopTimeoutMillis
import com.gemwallet.android.ui.models.StateViewType
import com.gemwallet.android.ui.models.navigation.requireAssetId
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.ChartPeriod
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.transformLatest
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.gemstone.AssetPrice
import uniffi.gemstone.GemChartPhase
import uniffi.gemstone.GemChartServiceInterface
import uniffi.gemstone.GemRefreshKind
import uniffi.gemstone.GemServiceException
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class ChartViewModel internal constructor(
    getCurrentCurrency: GetCurrentCurrency,
    getAssetTokenInfo: GetAssetTokenInfo,
    private val chartService: GemChartServiceInterface,
    private val assetId: AssetId,
    connectionStatusObserver: ConnectionStatusObserver,
    private val ioDispatcher: CoroutineDispatcher,
) : ViewModel() {
    private val selectedPeriod = MutableStateFlow(chartService.chartPeriod())
    private val session = MutableStateFlow(chartService.newSession())
    private val refreshRequests = MutableSharedFlow<Unit>(extraBufferCapacity = 1)

    val refreshIntervalMillis = connectionStatusObserver.refreshIntervalMillis(GemRefreshKind.CHART)
        .stateIn(viewModelScope, SharingStarted.Eagerly, 0L)

    private val price = getAssetTokenInfo(assetId)
        .map { info ->
            info?.price?.price?.let { price ->
                AssetPrice(
                    assetId = assetId.toIdentifier(),
                    price = price.price,
                    priceChangePercentage24h = price.priceChangePercentage24h,
                    updatedAt = price.updatedAt,
                )
            }
        }
        .distinctUntilChanged()

    private val loads = combine(
        selectedPeriod,
        getCurrentCurrency.getCurrency(),
        refreshRequests.onStart { emit(Unit) },
    ) { period, currency, _ -> period to currency }
        .transformLatest { (period, currency) ->
            session.update { held ->
                val kept = if (held.currency == currency.toGem()) held else chartService.newSession()
                kept.onSelectPeriod(period).onRefresh()
            }
            emit(Unit)
            try {
                val chart = chartService.syncCharts(assetId.toIdentifier(), period)
                session.update { held -> held.onLoaded(chart, period) }
            } catch (e: GemServiceException) {
                session.update { held -> held.onFailed(e, period) }
            }
            emit(Unit)
        }
        .flowOn(ioDispatcher)

    private val viewState = combine(session, price, loads) { session, price, _ -> session.viewState(price) }
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
                is GemChartPhase.Failed -> StateViewType.Error()
            },
        )
    }
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(StopTimeoutMillis), ChartUIModel.State())

    fun setPeriod(period: ChartPeriod) {
        if (period.toGem() == selectedPeriod.value) {
            return
        }
        viewModelScope.launch(ioDispatcher) {
            runCatchingCancellable { chartService.setChartPeriod(period.toGem()) }
                .onFailure { Log.e(TAG, "saving the chart period failed", it) }
        }
        selectedPeriod.value = period.toGem()
    }

    fun refresh() {
        refreshRequests.tryEmit(Unit)
    }

    @Inject
    constructor(
        getCurrentCurrency: GetCurrentCurrency,
        getAssetTokenInfo: GetAssetTokenInfo,
        chartService: GemChartServiceInterface,
        savedStateHandle: SavedStateHandle,
        connectionStatusObserver: ConnectionStatusObserver,
        @IoDispatcher ioDispatcher: CoroutineDispatcher,
    ) : this(
        getCurrentCurrency = getCurrentCurrency,
        getAssetTokenInfo = getAssetTokenInfo,
        chartService = chartService,
        assetId = savedStateHandle.requireAssetId(),
        connectionStatusObserver = connectionStatusObserver,
        ioDispatcher = ioDispatcher,
    )
}

private const val TAG = "Chart"
