package com.gemwallet.android.ui.models.perpetual.autoclose

import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.buttonState
import com.wallet.core.primitives.TpslType
import uniffi.gemstone.AutocloseValidation
import uniffi.gemstone.GemFormattedNumber
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemLocalizedText
import uniffi.gemstone.GemPerpetualPositionRow
import uniffi.gemstone.GemValueTone

data class AutocloseUIModel(val positionRow: GemPerpetualPositionRow?, val priceRows: List<GemListRow>, val takeProfit: Field, val stopLoss: Field, val confirmEnabled: Boolean) {
    val buttonState: ButtonState
        get() = buttonState(enabled = confirmEnabled)

    data class Field(val type: TpslType, val isProfit: Boolean, val pnl: GemLocalizedText?, val pnlDirection: GemValueTone, val percentSuggestions: List<GemFormattedNumber>, val validation: AutocloseValidation) {
        val showError: Boolean get() = validation != AutocloseValidation.VALID
    }
}
