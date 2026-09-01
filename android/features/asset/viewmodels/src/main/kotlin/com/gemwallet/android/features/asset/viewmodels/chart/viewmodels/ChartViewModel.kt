package com.gemwallet.android.features.asset.viewmodels.chart.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.assets.cases.GetAssetChartData
import com.gemwallet.android.application.assets.cases.GetAssetTokenInfo
import com.gemwallet.android.application.assets.cases.GetChartPeriod
import com.gemwallet.android.application.assets.cases.SetChartPeriod
import com.gemwallet.android.application.session.cases.GetCurrentCurrency
import com.gemwallet.android.features.asset.viewmodels.chart.models.AssetChartState
import com.gemwallet.android.features.asset.viewmodels.chart.models.ChartUIModel
import com.gemwallet.android.features.asset.viewmodels.chart.models.MinChartPoints
import com.gemwallet.android.features.asset.viewmodels.chart.models.StopTimeoutMillis
import com.gemwallet.android.features.asset.viewmodels.chart.models.from
import com.gemwallet.android.ui.models.StateViewType
import com.gemwallet.android.ui.models.flatMap
import com.gemwallet.android.ui.models.navigation.requireAssetId
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.ChartValue
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
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.transformLatest
import kotlinx.coroutines.flow.stateIn
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class ChartViewModel internal constructor(
    getAssetTokenInfo: GetAssetTokenInfo,
    getCurrentCurrency: GetCurrentCurrency,
    private val getAssetChartData: GetAssetChartData,
    getChartPeriod: GetChartPeriod,
    private val setChartPeriod: SetChartPeriod,
    private val assetId: AssetId,
) : ViewModel() {
    private val assetPriceInfo = getAssetTokenInfo(assetId)
        .map { it?.price }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)
    private val selectedPeriod = MutableStateFlow(getChartPeriod())
    private val refreshController = ChartRefreshController()

    val isRefreshing = refreshController.isRefreshing

    private val chartPrices = combine(
        selectedPeriod,
        getCurrentCurrency.getCurrency().distinctUntilChanged(),
        refreshController.trigger,
    ) { period, currency, _ -> AssetChartState(period, currency) }
        .transformLatest { state ->
            emit(state)
            val prices = try {
                request(state.period, state.currency)
            } catch (e: Exception) {
                currentCoroutineContext().ensureActive()
                null
            }
            refreshController.stopRefreshing()
            val chartPrices = when {
                prices == null -> StateViewType.Error
                prices.size < MinChartPoints -> StateViewType.NoData
                else -> StateViewType.Data(prices)
            }
            emit(state.copy(prices = chartPrices))
        }
        .flowOn(Dispatchers.IO)
        .stateIn(
            viewModelScope,
            SharingStarted.WhileSubscribed(StopTimeoutMillis),
            AssetChartState(selectedPeriod.value, Currency.USD),
        )

    val chartUIState = combine(assetPriceInfo, chartPrices) { priceInfo, state ->
        ChartUIModel.State(
            period = state.period,
            chart = state.prices.flatMap {
                StateViewType.Data(ChartUIModel.from(it, priceInfo, state.period, state.currency))
            },
        )
    }.stateIn(viewModelScope, SharingStarted.WhileSubscribed(StopTimeoutMillis), ChartUIModel.State())

    private suspend fun request(period: ChartPeriod, currency: Currency): List<ChartValue> {
        return getAssetChartData.getAssetChartData(assetId, period, currency)
    }

    fun setPeriod(period: ChartPeriod) {
        if (period == selectedPeriod.value) {
            return
        }
        viewModelScope.launch(Dispatchers.IO) { setChartPeriod(period) }
        selectedPeriod.value = period
    }

    fun refresh() {
        refreshController.startRefreshing()
    }

    @Inject
    constructor(
        getAssetTokenInfo: GetAssetTokenInfo,
        getCurrentCurrency: GetCurrentCurrency,
        getAssetChartData: GetAssetChartData,
        getChartPeriod: GetChartPeriod,
        setChartPeriod: SetChartPeriod,
        savedStateHandle: SavedStateHandle,
    ) : this(
        getAssetTokenInfo = getAssetTokenInfo,
        getCurrentCurrency = getCurrentCurrency,
        getAssetChartData = getAssetChartData,
        getChartPeriod = getChartPeriod,
        setChartPeriod = setChartPeriod,
        assetId = savedStateHandle.requireAssetId(),
    )

}
