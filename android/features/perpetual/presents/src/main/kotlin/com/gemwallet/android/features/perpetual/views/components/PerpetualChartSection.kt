package com.gemwallet.android.features.perpetual.views.components

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import com.gemwallet.android.domains.price.PriceChange
import com.gemwallet.android.math.getRelativeDate
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.model.NumericFormatter
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.chart.CandlestickTooltip
import com.gemwallet.android.ui.components.chart.ChartStateView
import com.gemwallet.android.ui.components.chart.GemCandlestickChart
import com.gemwallet.android.ui.models.chart.CandlestickChartUIModel
import com.gemwallet.android.ui.models.chart.CandlestickTooltipUIModel
import com.gemwallet.android.ui.models.chart.ChartHeaderUIModel
import com.gemwallet.android.ui.models.chart.ChartReferenceLineRole
import com.gemwallet.android.ui.models.chart.ChartReferenceLineUIModel
import com.gemwallet.android.ui.models.chart.ChartViewState
import com.gemwallet.android.ui.theme.paddingSmall
import com.wallet.core.primitives.ChartCandleStick
import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.Currency

private val TooltipRightSafeArea = 96.dp

@Composable
internal fun PerpetualChartSection(
    data: List<ChartCandleStick>,
    chartState: ChartViewState,
    period: ChartPeriod,
    entry: Double? = null,
    liquidation: Double? = null,
    stopLoss: Double? = null,
    takeProfit: Double? = null,
    onPeriodSelect: (ChartPeriod) -> Unit,
    modifier: Modifier = Modifier,
) {
    var selectedIndex by remember(period) { mutableStateOf<Int?>(null) }
    val safeSelectedIndex = selectedIndex?.takeIf { it in data.indices }
    val selectedCandle = safeSelectedIndex?.let { data[it] }
    val baseCandle = data.firstOrNull()
    val lastCandle = data.lastOrNull()
    val isSelectedRightHalf = safeSelectedIndex?.let { it.toFloat() / data.size.toFloat() > 0.5f } ?: false

    val currencyFormatter = remember { CurrencyFormatter(type = CurrencyFormatter.Type.Currency, currency = Currency.USD) }
    val numericFormatter = remember { NumericFormatter() }
    val volumeFormatter = remember { CurrencyFormatter(type = CurrencyFormatter.Type.Abbreviated, currency = Currency.USD) }
    val currencyString: (Double) -> String = remember(currencyFormatter) { currencyFormatter::string }
    val numericString: (Double) -> String = remember(numericFormatter) { { numericFormatter.string(it) } }
    val volumeString: (Double) -> String = remember(volumeFormatter) { volumeFormatter::string }

    val entryLabel = stringResource(R.string.charts_entry)
    val liquidationLabel = stringResource(R.string.perpetual_liquidation)
    val stopLossLabel = stringResource(R.string.perpetual_stop_loss)
    val takeProfitLabel = stringResource(R.string.perpetual_take_profit)
    val referenceLines = remember(entry, liquidation, stopLoss, takeProfit, entryLabel, liquidationLabel, stopLossLabel, takeProfitLabel, numericString) {
        listOfNotNull(
            entry?.let { ChartReferenceLineUIModel(it, "$entryLabel | ${numericString(it)}", ChartReferenceLineRole.Entry) },
            liquidation?.let { ChartReferenceLineUIModel(it, "$liquidationLabel | ${numericString(it)}", ChartReferenceLineRole.Liquidation) },
            stopLoss?.let { ChartReferenceLineUIModel(it, "$stopLossLabel | ${numericString(it)}", ChartReferenceLineRole.StopLoss) },
            takeProfit?.let { ChartReferenceLineUIModel(it, "$takeProfitLabel | ${numericString(it)}", ChartReferenceLineRole.TakeProfit) },
        )
    }

    val chartUIModel = remember(data, referenceLines, numericString) {
        if (data.isEmpty()) null
        else CandlestickChartUIModel.from(
            candles = data,
            yTickFormatter = numericString,
            referenceLines = referenceLines,
        )
    }
    val headerUIModel = remember(selectedCandle, baseCandle, lastCandle, currencyString) {
        val target = selectedCandle ?: lastCandle ?: return@remember null
        val base = baseCandle ?: return@remember null
        ChartHeaderUIModel.build(
            price = target.close,
            priceChangePercentage = PriceChange.percentage(from = base.close, to = target.close),
            timestamp = selectedCandle?.date,
            priceFormatter = currencyString,
            dateFormatter = ::getRelativeDate,
        )
    }
    val tooltipUIModel = remember(selectedCandle, numericString, volumeString) {
        selectedCandle?.let { CandlestickTooltipUIModel.from(it, numericString, volumeString) }
    }

    ChartStateView(
        state = chartState,
        header = headerUIModel,
        period = period,
        onPeriodSelect = onPeriodSelect,
        modifier = modifier,
    ) {
        if (chartUIModel != null) {
            GemCandlestickChart(
                model = chartUIModel,
                selectedIndex = safeSelectedIndex,
                onSelectionChanged = { selectedIndex = it },
            )
            TooltipOverlay(
                visible = tooltipUIModel != null,
                tooltip = tooltipUIModel,
                alignToStart = isSelectedRightHalf,
            )
        }
    }
}

@Composable
private fun BoxScope.TooltipOverlay(
    visible: Boolean,
    tooltip: CandlestickTooltipUIModel?,
    alignToStart: Boolean,
) {
    AnimatedVisibility(
        visible = visible,
        enter = fadeIn(),
        exit = fadeOut(),
        modifier = Modifier
            .align(if (alignToStart) Alignment.TopStart else Alignment.TopEnd)
            .padding(
                start = paddingSmall,
                end = if (alignToStart) paddingSmall else TooltipRightSafeArea,
                top = paddingSmall,
            ),
    ) {
        tooltip?.let { CandlestickTooltip(model = it) }
    }
}
