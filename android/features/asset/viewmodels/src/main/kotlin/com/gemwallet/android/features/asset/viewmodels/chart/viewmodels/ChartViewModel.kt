package com.gemwallet.android.features.asset.viewmodels.chart.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.assets.coordinators.GetAssetChartData
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.features.asset.viewmodels.assetIdArg
import com.gemwallet.android.features.asset.viewmodels.chart.models.ChartUIModel
import com.gemwallet.android.features.asset.viewmodels.chart.models.from
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.ChartValue
import com.wallet.core.primitives.Currency
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class ChartViewModel internal constructor(
    assetsRepository: AssetsRepository,
    sessionRepository: SessionRepository,
    private val getAssetChartData: GetAssetChartData,
    private val assetId: AssetId,
) : ViewModel() {
    private val assetPriceInfo = assetsRepository.getTokenInfo(assetId)
        .map { it?.price }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)
    private val selectedPeriod = MutableStateFlow(ChartPeriod.Day)
    private val chartState = MutableStateFlow(ChartState())
    private val refreshTrigger = MutableStateFlow(0L)
    private val refreshState = MutableStateFlow(false)

    val chartUIState = combine(selectedPeriod, chartState) { period, state ->
        ChartUIModel.State(state.loading, period, state.empty)
    }.stateIn(viewModelScope, SharingStarted.Eagerly, ChartUIModel.State())

    val isRefreshing = refreshState.asStateFlow()

    private val chartPrices = combine(
        selectedPeriod,
        sessionRepository.getCurrency().distinctUntilChanged(),
        refreshTrigger,
    ) { period, currency, _ -> period to currency }
        .mapLatest { (period, currency) ->
            try {
                val prices = request(period, currency)
                chartState.update { it.copy(loading = false, empty = prices.isEmpty()) }
                prices
            } catch (_: Exception) {
                currentCoroutineContext().ensureActive()
                chartState.update { it.copy(loading = false, empty = true) }
                emptyList()
            } finally {
                refreshState.value = false
            }
        }
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5_000), emptyList())

    val chartUIModel = combine(
        assetPriceInfo,
        selectedPeriod,
        chartPrices,
        sessionRepository.getCurrency().distinctUntilChanged(),
    ) { priceInfo, period, prices, currency ->
        ChartUIModel.from(prices, priceInfo, period, currency)
    }.stateIn(viewModelScope, SharingStarted.WhileSubscribed(5_000), ChartUIModel())

    private suspend fun request(period: ChartPeriod, currency: Currency): List<ChartValue> {
        return getAssetChartData.getAssetChartData(assetId, period, currency)
    }

    fun setPeriod(period: ChartPeriod) {
        if (period == selectedPeriod.value) {
            return
        }
        selectedPeriod.value = period
        chartState.update { ChartState(loading = true, empty = false) }
    }

    fun refresh() {
        refreshState.value = true
        chartState.update { ChartState(loading = true, empty = false) }
        refreshTrigger.value = refreshTrigger.value + 1
    }

    @Inject
    constructor(
        assetsRepository: AssetsRepository,
        sessionRepository: SessionRepository,
        getAssetChartData: GetAssetChartData,
        savedStateHandle: SavedStateHandle,
    ) : this(
        assetsRepository = assetsRepository,
        sessionRepository = sessionRepository,
        getAssetChartData = getAssetChartData,
        assetId = checkNotNull(savedStateHandle.get<String>(assetIdArg)?.toAssetId()),
    )

    private data class ChartState(
        val loading: Boolean = true,
        val empty: Boolean = false,
    )
}
