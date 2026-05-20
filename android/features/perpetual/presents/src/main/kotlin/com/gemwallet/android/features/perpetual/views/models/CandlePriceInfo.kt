package com.gemwallet.android.features.perpetual.views.models

import com.gemwallet.android.domains.percentage.formatAsPercentage
import com.gemwallet.android.domains.price.ValueDirection
import com.gemwallet.android.domains.price.toValueDirection
import com.gemwallet.android.model.CurrencyFormatter
import com.wallet.core.primitives.ChartCandleStick
import com.wallet.core.primitives.Currency

data class CandlePriceInfo(
    val priceValue: String,
    val changedPercentages: String,
    val state: ValueDirection,
) {
    companion object {
        private val priceFormatter = CurrencyFormatter(type = CurrencyFormatter.Type.Currency, currency = Currency.USD)

        fun from(point: ChartCandleStick?, data: List<ChartCandleStick>): CandlePriceInfo? {
            val base = data.firstOrNull()?.close
            if (point == null || base == null || base == 0.0) return null
            val periodChange = (point.close / base - 1.0) * 100.0
            return CandlePriceInfo(
                priceValue = priceFormatter.string(point.close),
                changedPercentages = periodChange.formatAsPercentage(),
                state = periodChange.toValueDirection(),
            )
        }
    }
}
