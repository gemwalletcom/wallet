package com.gemwallet.android.features.perpetual.viewmodels.models

import android.content.Context
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.perpetual.viewmodels.localization.stringRes
import com.gemwallet.android.ui.format.SectionDateFormatter
import com.gemwallet.android.ui.models.chart.CandlestickChartUIModel
import com.wallet.core.primitives.ChartCandleStick
import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.PerpetualPosition
import uniffi.gemstone.GemCandleViewState
import uniffi.gemstone.GemChartHeader
import uniffi.gemstone.chartDateStyle
import uniffi.gemstone.perpetualChartLayout
import java.time.ZoneId
import java.util.Locale

data class PerpetualChartUIModel(val candles: List<ChartCandleStick>, val state: GemCandleViewState, val chart: CandlestickChartUIModel) {
    fun header(selectedIndex: Int?): GemChartHeader? {
        val target = selectedIndex?.let { candles.getOrNull(it) } ?: candles.lastOrNull() ?: return null
        return state.headerAt(target.close)
    }

    fun dateText(selectedIndex: Int?, period: ChartPeriod, formatter: SectionDateFormatter): String? {
        val selected = selectedIndex?.let { candles.getOrNull(it) } ?: return null
        return formatter.chartDate(selected.date, chartDateStyle(period.toGem()), ZoneId.systemDefault(), Locale.getDefault())
    }

    companion object {
        fun from(state: GemCandleViewState, position: PerpetualPosition?, context: Context): PerpetualChartUIModel {
            val candles = state.viewport.candles.map { it.toPrimitives() }
            return PerpetualChartUIModel(
                candles = candles,
                state = state,
                chart = CandlestickChartUIModel.from(
                    candles = candles,
                    layout = perpetualChartLayout(state.viewport.candles, position?.toGem()),
                    lineLabel = { kind -> context.getString(kind.stringRes()) },
                ),
            )
        }
    }
}
