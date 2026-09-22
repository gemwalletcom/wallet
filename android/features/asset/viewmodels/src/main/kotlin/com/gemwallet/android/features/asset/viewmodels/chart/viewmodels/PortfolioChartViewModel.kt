package com.gemwallet.android.features.asset.viewmodels.chart.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.gemstone.connection.ConnectionStatusObserver
import com.gemwallet.android.data.services.gemstone.perpetual.ObservePerpetualWallet
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.asset.viewmodels.chart.models.ChartUIModel
import com.gemwallet.android.features.asset.viewmodels.chart.models.StopTimeoutMillis
import com.gemwallet.android.ui.models.StateViewType
import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PortfolioType
import com.wallet.core.primitives.Wallet
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.distinctUntilChangedBy
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemPortfolioPhase
import uniffi.gemstone.GemPortfolioServiceInterface
import uniffi.gemstone.GemPortfolioSession
import uniffi.gemstone.GemPortfolioViewState
import uniffi.gemstone.GemRefreshKind
import uniffi.gemstone.PortfolioChartType
import uniffi.gemstone.portfolioSession
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class PortfolioChartViewModel internal constructor(
    private val service: GemPortfolioServiceInterface,
    getSession: GetSession,
    observePerpetualWallet: ObservePerpetualWallet,
    initialType: PortfolioType,
    connectionStatusObserver: ConnectionStatusObserver,
    private val ioDispatcher: CoroutineDispatcher,
) : ViewModel() {

    private val wallet = MutableStateFlow<Wallet?>(null)
    private val session = MutableStateFlow(portfolioSession(initialType.toGem()))

    private val viewState: StateFlow<GemPortfolioViewState> = session
        .map { it.viewState() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, session.value.viewState())

    val selectedType: StateFlow<PortfolioType> = viewState
        .map { it.portfolioType.toPrimitives() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, initialType)

    val selectedChartType: StateFlow<PortfolioChartType> = viewState
        .map { it.chartType }
        .stateIn(viewModelScope, SharingStarted.Eagerly, viewState.value.chartType)

    val availablePeriods: StateFlow<List<ChartPeriod>> = viewState
        .map { state -> state.periods.map { it.toPrimitives() } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, viewState.value.periods.map { it.toPrimitives() })

    val statistics: StateFlow<List<GemListRow>> = viewState
        .map { it.statistics }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val currency: StateFlow<Currency> = viewState
        .map { it.currency.toPrimitives() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, Currency.USD)

    val isRefreshing: StateFlow<Boolean> = viewState
        .map { it.isRefreshing }
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    val chartUIState: StateFlow<ChartUIModel.State> = viewState
        .map { state -> ChartUIModel.State(period = state.period.toPrimitives(), chart = state.phase.chartState()) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, ChartUIModel.State())

    val showSegmentedControl = observePerpetualWallet()
        .map { it != null }
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(StopTimeoutMillis), false)

    val refreshIntervalMillis = connectionStatusObserver.refreshIntervalMillis(GemRefreshKind.CHART)
        .stateIn(viewModelScope, SharingStarted.Eagerly, 0L)

    init {
        viewModelScope.launch {
            getSession().filterNotNull().distinctUntilChangedBy { it.wallet.id.id to it.currency }.collect { current ->
                wallet.update { current.wallet }
                session.update { it.onSelectWallet(current.wallet.id.id, current.currency.toGem()) }
            }
        }
        viewModelScope.launch {
            session.distinctUntilChangedBy { it.request() to it.needsLoad() }
                .collectLatest { if (it.needsLoad()) load() }
        }
    }

    fun setType(type: PortfolioType) = session.update { it.onSelectType(type.toGem()) }

    fun setChartType(chartType: PortfolioChartType) = session.update { it.onSelectChartType(chartType) }

    fun setPeriod(period: ChartPeriod) = session.update { it.onSelectPeriod(period.toGem()) }

    fun refresh() {
        session.update { it.onRefresh() }
    }

    private suspend fun load() {
        val current = wallet.value ?: return
        val result = withContext(ioDispatcher) { service.refresh(current.toGem(), session.value.request()) }
        session.update { it.onResult(result) }
    }

    private fun GemPortfolioPhase.chartState(): StateViewType<ChartUIModel> = when (this) {
        GemPortfolioPhase.Loading -> StateViewType.Loading
        is GemPortfolioPhase.Data -> StateViewType.Data(ChartUIModel(chart = chart))
        GemPortfolioPhase.NoData -> StateViewType.NoData
        is GemPortfolioPhase.Failed -> StateViewType.Error()
    }

    @Inject
    constructor(
        service: GemPortfolioServiceInterface,
        getSession: GetSession,
        observePerpetualWallet: ObservePerpetualWallet,
        savedStateHandle: SavedStateHandle,
        connectionStatusObserver: ConnectionStatusObserver,
        @IoDispatcher ioDispatcher: CoroutineDispatcher,
    ) : this(
        service = service,
        getSession = getSession,
        observePerpetualWallet = observePerpetualWallet,
        initialType = savedStateHandle.portfolioType(),
        connectionStatusObserver = connectionStatusObserver,
        ioDispatcher = ioDispatcher,
    )
}
