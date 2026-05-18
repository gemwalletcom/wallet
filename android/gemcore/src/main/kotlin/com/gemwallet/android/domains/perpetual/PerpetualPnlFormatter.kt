package com.gemwallet.android.domains.perpetual

import com.gemwallet.android.domains.percentage.formatAsPercentage
import com.gemwallet.android.model.formatPnl
import com.wallet.core.primitives.Currency

fun formatPnlWithPercentage(pnl: Double, marginAmount: Double, dynamicPlace: Boolean = false): String {
    val percentage = if (marginAmount > 0.0) {
        (pnl / marginAmount) * 100.0
    } else {
        0.0
    }.formatAsPercentage()
    return "${Currency.USD.formatPnl(pnl, dynamicPlace = dynamicPlace)} ($percentage)"
}
