package com.gemwallet.android.ui.models.perpetual

import com.gemwallet.android.domains.price.ValueDirection
import com.wallet.core.primitives.PerpetualDirection

data class PerpetualConfirmDetailsUIModel(
    val action: Action,
    val direction: PerpetualDirection,
    val leverage: Int,
    val pnl: Pnl?,
    val marginText: String,
    val sizeText: String,
    val autoclose: Autoclose?,
    val marketPriceText: String,
    val entryPriceText: String?,
    val slippageText: String,
) {
    enum class Action { Open, Close, Increase, Reduce }

    data class Pnl(
        val text: String,
        val direction: ValueDirection,
    )

    data class Autoclose(
        val takeProfitText: String?,
        val stopLossText: String?,
    )
}
