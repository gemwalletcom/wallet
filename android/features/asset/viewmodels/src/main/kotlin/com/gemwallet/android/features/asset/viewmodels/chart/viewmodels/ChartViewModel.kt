package com.gemwallet.android.features.asset.viewmodels.chart.viewmodels

import android.text.format.DateUtils
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.services.gemapi.GemApiClient
import com.gemwallet.android.domains.percentage.formatAsPercentage
import com.gemwallet.android.domains.price.toPriceState
import com.gemwallet.android.ext.tickerFlow
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.AssetPriceInfo
import com.gemwallet.android.model.format
import com.gemwallet.android.features.asset.viewmodels.assetIdArg
import com.gemwallet.android.features.asset.viewmodels.chart.models.ChartUIModel
import com.gemwallet.android.features.asset.viewmodels.chart.models.PricePoint
import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.ChartValue
import com.wallet.core.primitives.Currency
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class ChartViewModel @Inject constructor(
    private val assetsRepository: AssetsRepository,
    private val gemApiClient: GemApiClient,
    savedStateHandle: SavedStateHandle,
) : ViewModel() {
    private val assetIdStr = savedStateHandle.getStateFlow<String?>(assetIdArg, null)
    private val assetInfo = assetIdStr
        .flatMapLatest {
            val assetId = it?.toAssetId() ?: return@flatMapLatest emptyFlow()
            assetsRepository.getTokenInfo(assetId).filterNotNull()
        }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)
    private val selectedPeriod = MutableStateFlow(ChartPeriod.Day)
    private val chartState = MutableStateFlow(ChartState())
    val chartUIState = combine(selectedPeriod, chartState) { period, state ->
        ChartUIModel.State(state.loading, period, state.empty)
    }.stateIn(viewModelScope, SharingStarted.Eagerly, ChartUIModel.State())

    private val ticker = tickerFlow(DateUtils.MINUTE_IN_MILLIS) {}
        .stateIn(viewModelScope, SharingStarted.Eagerly, 0L)

    val chartUIModel = combine(assetInfo, selectedPeriod, ticker) { assetInfo, period, _ ->
        assetInfo to period
    }
        .mapLatest { (assetInfo, period) ->
            val assetInfo = assetInfo ?: return@mapLatest ChartUIModel(period = period)
            try {
                val model = request(assetInfo, period)
                chartState.update { it.copy(loading = false, empty = model.chartPoints.isEmpty()) }
                model
            } catch (e: CancellationException) {
                throw e
            } catch (_: Throwable) {
                chartState.update { it.copy(loading = false, empty = true) }
                ChartUIModel(period = period)
            }
        }
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5_000), ChartUIModel())

    private suspend fun request(assetInfo: AssetInfo, period: ChartPeriod): ChartUIModel {
        val currency = assetInfo.price?.currency ?: Currency.USD

        val prices = try {
            val rate = assetsRepository.getCurrencyRate(currency).firstOrNull()?.rate?.toFloat() ?: throw IllegalStateException()
            gemApiClient.getChart(assetInfo.asset.id.toIdentifier(), period.string).prices.map {
                it.copy(value = it.value * rate)
            }.sortedBy { it.timestamp }
        } catch (e: CancellationException) {
            throw e
        } catch (_: Throwable) {
            emptyList()
        }

        return buildChartUIModel(prices, assetInfo.price, period, currency)
    }

    fun setPeriod(period: ChartPeriod) {
        if (period == selectedPeriod.value) {
            return
        }
        selectedPeriod.value = period
        chartState.update { ChartState(loading = true, empty = false) }
    }

    private data class ChartState(
        val loading: Boolean = true,
        val empty: Boolean = false,
    )
}

internal fun buildChartUIModel(
    prices: List<ChartValue>,
    priceInfo: AssetPriceInfo?,
    period: ChartPeriod,
    currency: Currency,
): ChartUIModel {
    val periodStartPrice = prices.firstOrNull { it.value != 0.0f }?.value ?: prices.firstOrNull()?.value ?: 0.0f
    val historicalPoints = prices.map {
        val percent = percentageChange(periodStartPrice, it.value.toDouble())
        PricePoint(
            y = it.value,
            yLabel = currency.format(it.value, 2, dynamicPlace = true),
            timestamp = it.timestamp * 1000L,
            percentage = percent.formatAsPercentage(),
            priceState = percent.toPriceState(),
        )
    }
    val lastTimestampMillis = (prices.lastOrNull()?.timestamp ?: 0) * 1000L
    val currentPoint = priceInfo
        ?.takeIf { historicalPoints.isNotEmpty() && it.price.updatedAt > lastTimestampMillis }
        ?.let {
            val percentage = if (period == ChartPeriod.Day) {
                it.price.priceChangePercentage24h
            } else {
                percentageChange(periodStartPrice, it.price.price)
            }
            PricePoint(
                y = it.price.price.toFloat(),
                yLabel = currency.format(it.price.price, dynamicPlace = true),
                timestamp = System.currentTimeMillis(),
                percentage = percentage.formatAsPercentage(),
                priceState = percentage.toPriceState(),
            )
        }
    val chartPoints = historicalPoints + listOfNotNull(currentPoint)

    return ChartUIModel(
        period = period,
        currentPoint = currentPoint,
        chartPoints = chartPoints,
    )
}

private fun percentageChange(periodStartPrice: Float, currentPrice: Double): Double {
    if (periodStartPrice == 0.0f) {
        return 0.0
    }
    return (currentPrice - periodStartPrice) / periodStartPrice * 100.0
}
