package com.gemwallet.android.features.asset.viewmodels.chart.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.assets.coordinators.GetPortfolioData
import com.gemwallet.android.application.assets.coordinators.walletChartPeriods
import com.gemwallet.android.application.session.coordinators.GetCurrentCurrency
import com.gemwallet.android.data.repositories.perpetual.ObservePerpetualWallet
import com.gemwallet.android.features.asset.viewmodels.chart.models.ChartUIModel
import com.gemwallet.android.features.asset.viewmodels.chart.models.PortfolioState
import com.gemwallet.android.features.asset.viewmodels.chart.models.StopTimeoutMillis
import com.gemwallet.android.features.asset.viewmodels.chart.models.chartValues
import com.gemwallet.android.features.asset.viewmodels.chart.models.from
import com.gemwallet.android.features.asset.viewmodels.chart.models.hasVariation
import com.gemwallet.android.ui.models.StateViewType
import com.gemwallet.android.ui.models.dataOrNull
import com.gemwallet.android.ui.models.flatMap
import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PortfolioChartType
import com.wallet.core.primitives.PortfolioType
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.transformLatest
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class PortfolioChartViewModel internal constructor(
    getCurrentCurrency: GetCurrentCurrency,
    observePerpetualWallet: ObservePerpetualWallet,
    private val getPortfolioData: GetPortfolioData,
    initialType: PortfolioType,
) : ViewModel() {
    private val _selectedType = MutableStateFlow(initialType)
    val selectedType = _selectedType.asStateFlow()

    private val _selectedChartType = MutableStateFlow(PortfolioChartType.Pnl)
    val selectedChartType = _selectedChartType.asStateFlow()

    private val selectedPeriod = MutableStateFlow(ChartPeriod.All)
    private val refreshTrigger = MutableStateFlow(0L)
    private val refreshState = MutableStateFlow(false)

    val showSegmentedControl = observePerpetualWallet()
        .map { it != null }
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(StopTimeoutMillis), false)

    val isRefreshing = refreshState.asStateFlow()

    private val portfolio = combine(
        selectedType,
        selectedPeriod,
        getCurrentCurrency.getCurrency().distinctUntilChanged(),
        refreshTrigger,
    ) { type, period, currency, _ -> PortfolioState(type, period, currency) }
        .transformLatest { state ->
            emit(state)
            val data = try {
                getPortfolioData.getPortfolioData(state.type, state.period, state.currency)
            } catch (e: Exception) {
                currentCoroutineContext().ensureActive()
                null
            }
            refreshState.value = false
            val periods = data?.availablePeriods.orEmpty()
            when {
                data == null -> emit(state.copy(data = StateViewType.Error))
                periods.isNotEmpty() && !periods.contains(state.period) ->
                    selectedPeriod.compareAndSet(state.period, periods.first())
                else -> emit(state.copy(data = StateViewType.Data(data)))
            }
        }
        .flowOn(Dispatchers.IO)
        .stateIn(
            viewModelScope,
            SharingStarted.WhileSubscribed(StopTimeoutMillis),
            PortfolioState(initialType, selectedPeriod.value, Currency.USD),
        )

    val chartUIState = combine(portfolio, selectedChartType) { state, chartType ->
        ChartUIModel.State(
            period = state.period,
            chart = state.data.flatMap { data ->
                val values = data.chartValues(chartType)
                if (values.hasVariation()) {
                    StateViewType.Data(
                        ChartUIModel.from(
                            values = values,
                            period = state.period,
                            currency = state.displayCurrency(),
                            showHeaderValue = state.type == PortfolioType.Wallet || chartType == PortfolioChartType.Value,
                        ),
                    )
                } else {
                    StateViewType.NoData
                }
            },
        )
    }.stateIn(viewModelScope, SharingStarted.WhileSubscribed(StopTimeoutMillis), ChartUIModel.State())

    val statistics = portfolio
        .map { it.data.dataOrNull?.statistics.orEmpty() }
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(StopTimeoutMillis), emptyList())

    val currency = portfolio
        .map { it.displayCurrency() }
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(StopTimeoutMillis), Currency.USD)

    val availablePeriods = portfolio
        .map { it.data.dataOrNull?.availablePeriods?.takeIf { periods -> periods.isNotEmpty() } ?: walletChartPeriods }
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(StopTimeoutMillis), walletChartPeriods)

    fun setType(type: PortfolioType) {
        _selectedType.value = type
    }

    fun setChartType(chartType: PortfolioChartType) {
        _selectedChartType.value = chartType
    }

    fun setPeriod(period: ChartPeriod) {
        selectedPeriod.value = period
    }

    fun refresh() {
        refreshState.value = true
        refreshTrigger.value = refreshTrigger.value + 1
    }

    @Inject
    constructor(
        getCurrentCurrency: GetCurrentCurrency,
        observePerpetualWallet: ObservePerpetualWallet,
        getPortfolioData: GetPortfolioData,
        savedStateHandle: SavedStateHandle,
    ) : this(
        getCurrentCurrency = getCurrentCurrency,
        observePerpetualWallet = observePerpetualWallet,
        getPortfolioData = getPortfolioData,
        initialType = savedStateHandle.portfolioType(),
    )
}

private fun PortfolioState.displayCurrency(): Currency =
    if (type == PortfolioType.Perpetuals) Currency.USD else currency
