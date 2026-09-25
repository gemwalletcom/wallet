package com.gemwallet.android.features.perpetual.viewmodels.models

import android.content.Context
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ui.format.SectionDateFormatter
import com.gemwallet.android.ui.localization.stringRes
import com.gemwallet.android.ui.models.chart.CandlestickChartUIModel
import com.wallet.core.primitives.ChartCandleStick
import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.PerpetualPosition
import uniffi.gemstone.GemChartHeader
import uniffi.gemstone.candlestickHeader
import uniffi.gemstone.chartDateStyle
import uniffi.gemstone.perpetualChartLayout
import java.time.ZoneId
import java.util.Locale

data class PerpetualChartUIModel(val candles: List<ChartCandleStick>, val chart: CandlestickChartUIModel) {
    fun header(selectedIndex: Int?): GemChartHeader? {
        val base = candles.firstOrNull() ?: return null
        val target = selectedIndex?.let { candles.getOrNull(it) } ?: candles.last()
        return candlestickHeader(base.close, target.close)
    }

    fun dateText(selectedIndex: Int?, period: ChartPeriod, formatter: SectionDateFormatter): String? {
        val selected = selectedIndex?.let { candles.getOrNull(it) } ?: return null
        return formatter.chartDate(selected.date, chartDateStyle(period.toGem()), ZoneId.systemDefault(), Locale.getDefault())
    }

    companion object {
        fun from(candles: List<ChartCandleStick>, position: PerpetualPosition?, context: Context): PerpetualChartUIModel = PerpetualChartUIModel(
            candles = candles,
            chart = CandlestickChartUIModel.from(
                candles = candles,
                layout = perpetualChartLayout(candles.map { it.toGem() }, position?.toGem()),
                lineLabel = { kind -> context.getString(kind.stringRes()) },
            ),
        )
    }
}
