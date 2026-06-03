package com.gemwallet.android.ui.models.perpetual.autoclose

import com.gemwallet.android.domains.perpetual.aggregates.PerpetualPositionDataAggregate
import com.gemwallet.android.domains.perpetual.autoclose.AutocloseError
import com.gemwallet.android.domains.price.ValueDirection
import com.wallet.core.primitives.TpslType

data class AutocloseUIModel(
    val position: PerpetualPositionDataAggregate,
    val marketPriceText: String,
    val entryPriceText: String,
    val takeProfit: Field,
    val stopLoss: Field,
    val confirmEnabled: Boolean,
) {
    data class Field(
        val type: TpslType,
        val isProfit: Boolean,
        val pnlText: String,
        val pnlDirection: ValueDirection,
        val percentSuggestions: List<Int>,
        val error: AutocloseError?,
    ) {
        val showError: Boolean get() = error != null
    }
}
