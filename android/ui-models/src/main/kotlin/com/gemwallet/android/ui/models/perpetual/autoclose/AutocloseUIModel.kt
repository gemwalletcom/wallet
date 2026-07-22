package com.gemwallet.android.ui.models.perpetual.autoclose

import com.gemwallet.android.domains.perpetual.aggregates.PerpetualPositionDataAggregate
import com.gemwallet.android.domains.price.ValueDirection
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.buttonState
import com.wallet.core.primitives.TpslType
import uniffi.gemstone.AutocloseValidation

data class AutocloseUIModel(
    val position: PerpetualPositionDataAggregate,
    val marketPriceText: String,
    val entryPriceText: String,
    val takeProfit: Field,
    val stopLoss: Field,
    val confirmEnabled: Boolean,
) {
    val buttonState: ButtonState
        get() = buttonState(enabled = confirmEnabled)

    data class Field(
        val type: TpslType,
        val isProfit: Boolean,
        val pnlText: String,
        val pnlDirection: ValueDirection,
        val percentSuggestions: List<Int>,
        val validation: AutocloseValidation,
    ) {
        val showError: Boolean get() = validation != AutocloseValidation.VALID
    }
}
