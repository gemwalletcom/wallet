package com.gemwallet.android.data.coordinators.perpetuals

import com.gemwallet.android.application.perpetual.cases.PerpetualCandles
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.ChartCandleStick
import com.wallet.core.primitives.ChartPeriod
import uniffi.gemstone.GemPerpetualService

class PerpetualCandlesImpl(
    private val perpetualService: GemPerpetualService,
) : PerpetualCandles {

    override fun candleInterval(period: ChartPeriod): String = perpetualService.candleInterval(period.toJson())

    override fun mergeCandle(candles: List<ChartCandleStick>, candle: ChartCandleStick): List<ChartCandleStick> =
        perpetualService.mergeCandle(candles.map { it.toJson() }, candle.toJson()).map { it.decodeJson() }
}
