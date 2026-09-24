package com.gemwallet.android.ui.models.chart

import android.text.format.DateFormat
import com.gemwallet.android.ext.MILLIS_PER_SECOND
import com.gemwallet.android.model.text
import uniffi.gemstone.ChartCandleStick
import uniffi.gemstone.GemCandleTickFormat
import uniffi.gemstone.GemCandleViewport
import uniffi.gemstone.GemPerpetualChartLayout
import uniffi.gemstone.GemPerpetualChartLineKind
import uniffi.gemstone.GemValueTone
import java.time.Instant
import java.time.LocalTime
import java.time.ZoneId
import java.time.ZonedDateTime
import java.time.format.DateTimeFormatter
import java.time.format.FormatStyle
import java.util.Locale

enum class CandleDirection {
    Up,
    Down,
    Flat,
}

data class ChartReferenceLineUIModel(val kind: GemPerpetualChartLineKind, val price: Double, val overlapLevel: Int, val label: String)

data class ChartAxisTick(val value: Double, val fraction: Float, val label: String)

data class ChartTimeTick(val fraction: Float, val label: String)

data class CandleUIModel(val x: Float, val open: Double, val high: Double, val low: Double, val close: Double, val direction: CandleDirection)

data class CandlestickChartUIModel(
    val candles: List<CandleUIModel>,
    val bodyWidthFraction: Float,
    val yMin: Double,
    val yMax: Double,
    val yTicks: List<ChartAxisTick>,
    val xTicks: List<ChartTimeTick>,
    val referenceLines: List<ChartReferenceLineUIModel>,
    val currentPriceLabel: String,
) {
    val ySpan: Double get() = yMax - yMin

    companion object {
        private const val BODY_WIDTH_RATIO = 0.6f

        fun from(viewport: GemCandleViewport, layout: GemPerpetualChartLayout, lineLabel: (GemPerpetualChartLineKind) -> String, zone: ZoneId = ZoneId.systemDefault(), locale: Locale = Locale.getDefault()): CandlestickChartUIModel {
            val span = (viewport.end - viewport.start).coerceAtLeast(1L).toFloat()
            fun fraction(date: Long): Float = (date - viewport.start) / span
            val priceSpan = layout.priceHigh - layout.priceLow
            val labels = tickLabels(viewport.ticks.map { Instant.ofEpochMilli(it).atZone(zone) }, viewport.tickFormat, locale)
            return CandlestickChartUIModel(
                candles = viewport.candles.zip(layout.tones) { candle, tone -> candleUIModel(candle, tone, fraction(candle.date)) },
                bodyWidthFraction = viewport.intervalSeconds * MILLIS_PER_SECOND * BODY_WIDTH_RATIO / span,
                yMin = layout.priceLow,
                yMax = layout.priceHigh,
                yTicks = layout.ticks.map { tick ->
                    ChartAxisTick(value = tick.value, fraction = ((tick.value - layout.priceLow) / priceSpan).toFloat(), label = tick.text())
                },
                xTicks = viewport.ticks.zip(labels) { date, label -> ChartTimeTick(fraction = fraction(date), label = label) },
                referenceLines = layout.lines.map { line ->
                    ChartReferenceLineUIModel(
                        kind = line.kind,
                        price = line.price.value,
                        overlapLevel = line.overlapLevel.toInt(),
                        label = "${lineLabel(line.kind)} | ${line.price.text()}",
                    )
                },
                currentPriceLabel = layout.currentPrice?.text().orEmpty(),
            )
        }

        private fun candleUIModel(candle: ChartCandleStick, tone: GemValueTone, x: Float): CandleUIModel = CandleUIModel(
            x = x,
            open = candle.open,
            high = candle.high,
            low = candle.low,
            close = candle.close,
            direction = when (tone) {
                GemValueTone.POSITIVE -> CandleDirection.Up

                GemValueTone.NEGATIVE -> CandleDirection.Down

                GemValueTone.NEUTRAL,
                GemValueTone.PLAIN,
                GemValueTone.WARNING,
                -> CandleDirection.Flat
            },
        )
    }
}

private fun tickLabels(ticks: List<ZonedDateTime>, format: GemCandleTickFormat, locale: Locale): List<String> {
    val time = DateTimeFormatter.ofLocalizedTime(FormatStyle.SHORT).withLocale(locale)
    val day by lazy { bestPattern("dMMM", locale) }
    val monthYear by lazy { bestPattern("MMMy", locale) }
    return ticks.map { tick ->
        when (format) {
            GemCandleTickFormat.TIME -> time
            GemCandleTickFormat.TIME_OR_DAY -> if (tick.startsDay(ticks)) day else time
            GemCandleTickFormat.DAY -> day
            GemCandleTickFormat.MONTH_YEAR -> monthYear
        }.format(tick)
    }
}

private fun bestPattern(skeleton: String, locale: Locale): DateTimeFormatter = DateTimeFormatter.ofPattern(DateFormat.getBestDateTimePattern(locale, skeleton), locale)

private fun ZonedDateTime.startsDay(ticks: List<ZonedDateTime>): Boolean {
    val isFirstOfEarlierDay = toLocalDate() != ticks.last().toLocalDate() && ticks.first { it.toLocalDate() == toLocalDate() } == this
    return isFirstOfEarlierDay || toLocalTime() == LocalTime.MIDNIGHT
}
