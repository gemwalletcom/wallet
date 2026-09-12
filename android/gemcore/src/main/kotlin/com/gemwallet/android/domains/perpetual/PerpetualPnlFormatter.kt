package com.gemwallet.android.domains.perpetual

import com.gemwallet.android.domains.percentage.formatAsPercentage
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.model.PriceChangeFormatter
import com.gemwallet.android.domains.price.PriceChangeCalculator
import com.wallet.core.primitives.Currency

private val pnlFormatter = PriceChangeFormatter(
    CurrencyFormatter(type = CurrencyFormatter.Type.Currency, currency = Currency.USD)
)

fun formatPnlWithPercentage(pnl: Double, marginAmount: Double): String {
    val percentage = PriceChangeCalculator.pnlPercentage(pnl, marginAmount).formatAsPercentage()
    return "${pnlFormatter.string(pnl)} ($percentage)"
}
