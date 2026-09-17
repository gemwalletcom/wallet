package com.gemwallet.android.features.perpetual.viewmodels.models

import android.content.Context
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.features.perpetual.viewmodels.localization.stringRes
import com.gemwallet.android.ui.models.chart.CandlestickChartUIModel
import com.wallet.core.primitives.ChartCandleStick
import com.wallet.core.primitives.PerpetualPosition
import uniffi.gemstone.perpetualChartLayout

data class PerpetualChartUIModel(
    val candles: List<ChartCandleStick>,
    val chart: CandlestickChartUIModel,
) {
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
