package com.gemwallet.android.features.asset.viewmodels.chart.viewmodels

import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.ext.toGem
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.session.cases.GetCurrentCurrency
import com.gemwallet.android.features.asset.viewmodels.chart.models.AssetChartState
import com.gemwallet.android.features.asset.viewmodels.chart.models.ChartUIModel
import com.gemwallet.android.features.asset.viewmodels.chart.models.StopTimeoutMillis
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.ui.models.StateViewType
import com.gemwallet.android.ui.models.flatMap
import com.gemwallet.android.ui.models.navigation.requireAssetId
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.AssetId
import uniffi.gemstone.GemChartService
import uniffi.gemstone.GemChartServiceInterface
import uniffi.gemstone.priceChartData
import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.Currency
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
    private val selectedPeriod = MutableStateFlow(chartService.chartPeriod().toPrimitives())
    private val refreshController = ChartRefreshController()

    val isRefreshing = refreshController.isRefreshing

    private val chartPrices = combine(
        selectedPeriod,
        getCurrentCurrency.getCurrency(),
        refreshController.trigger,
    ) { period, currency, _ -> AssetChartState(period, currency) }
        .transformLatest { state ->
            emit(state)
            val chart = try {
                chartService.syncCharts(assetId.toIdentifier(), state.period.toGem())
            } catch (e: Exception) {
                currentCoroutineContext().ensureActive()
                null
            }
            refreshController.stopRefreshing()
            emit(state.copy(prices = chart?.let { StateViewType.Data(it) } ?: StateViewType.Error))
        }
        .flowOn(Dispatchers.IO)
        .stateIn(
            viewModelScope,
            SharingStarted.WhileSubscribed(StopTimeoutMillis),
            AssetChartState(selectedPeriod.value, Currency.USD),
        )

    val chartUIState = chartPrices.map { state ->
        val currencyFormatter = CurrencyFormatter(currency = state.currency)
        ChartUIModel.State(
            period = state.period,
            chart = state.prices.flatMap { chart ->
                priceChartData(chart)
                    ?.let { StateViewType.Data(ChartUIModel(it, currencyFormatter::string)) }
                    ?: StateViewType.NoData
            },
        )
    }
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(StopTimeoutMillis), ChartUIModel.State())

    fun setPeriod(period: ChartPeriod) {
        if (period == selectedPeriod.value) {
            return
        }
        viewModelScope.launch(Dispatchers.IO) { chartService.setChartPeriod(period.toGem()) }
        selectedPeriod.value = period
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
