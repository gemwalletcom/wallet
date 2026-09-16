package com.gemwallet.android.features.asset.viewmodels.chart.viewmodels

import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.ext.toGem
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.session.cases.GetCurrentCurrency
import com.gemwallet.android.features.asset.viewmodels.chart.models.ChartUIModel
import com.gemwallet.android.features.asset.viewmodels.chart.models.StopTimeoutMillis
import com.gemwallet.android.ui.models.StateViewType
import com.gemwallet.android.ui.models.navigation.requireAssetId
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.AssetId
import uniffi.gemstone.GemChartService
import uniffi.gemstone.GemChartServiceInterface
import uniffi.gemstone.GemChartPhase
import uniffi.gemstone.GemServiceException
import com.wallet.core.primitives.ChartPeriod
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.transformLatest
import kotlinx.coroutines.flow.stateIn
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class ChartViewModel internal constructor(
    getCurrentCurrency: GetCurrentCurrency,
    private val chartService: GemChartServiceInterface,
    private val assetId: AssetId,
) : ViewModel() {
    private val selectedPeriod = MutableStateFlow(chartService.chartPeriod())
    private val refreshController = ChartRefreshController()

    val isRefreshing = refreshController.isRefreshing

    private val loaded = combine(
        selectedPeriod,
        getCurrentCurrency.getCurrency(),
        refreshController.trigger,
    ) { period, _, _ -> period }
        .transformLatest { period ->
            val loading = chartService.newSession().onSelectPeriod(period)
            emit(loading.viewState())
            val next = try {
                loading.onLoaded(chartService.syncCharts(assetId.toIdentifier(), period))
            } catch (e: Exception) {
                currentCoroutineContext().ensureActive()
                loading.onFailed(e as? GemServiceException ?: GemServiceException.Core(e.message.orEmpty()))
            }
            refreshController.stopRefreshing()
            emit(next.viewState())
        }
        .flowOn(Dispatchers.IO)
        .stateIn(
            viewModelScope,
            SharingStarted.WhileSubscribed(StopTimeoutMillis),
            chartService.newSession().viewState(),
        )

    val chartUIState = loaded.map { state ->
        ChartUIModel.State(
            period = state.period.toPrimitives(),
            chart = when (val phase = state.phase) {
                GemChartPhase.Loading -> StateViewType.Loading
                is GemChartPhase.Data -> StateViewType.Data(ChartUIModel(phase.data))
                GemChartPhase.NoData -> StateViewType.NoData
                is GemChartPhase.Failed -> StateViewType.Error
            },
        )
    }
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(StopTimeoutMillis), ChartUIModel.State())

    fun setPeriod(period: ChartPeriod) {
        if (period.toGem() == selectedPeriod.value) {
            return
        }
        viewModelScope.launch(Dispatchers.IO) { chartService.setChartPeriod(period.toGem()) }
        selectedPeriod.value = period.toGem()
    }

    fun refresh() {
        refreshController.startRefreshing()
    }

    @Inject
    constructor(
        getCurrentCurrency: GetCurrentCurrency,
        chartService: GemChartService,
        savedStateHandle: SavedStateHandle,
    ) : this(
        getCurrentCurrency = getCurrentCurrency,
        chartService = chartService,
        assetId = savedStateHandle.requireAssetId(),
    )

}
