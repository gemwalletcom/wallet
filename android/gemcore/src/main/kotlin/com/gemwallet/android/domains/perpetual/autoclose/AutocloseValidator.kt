package com.gemwallet.android.domains.perpetual.autoclose

import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.TpslType

class AutocloseValidator(
    private val type: TpslType,
    private val direction: PerpetualDirection,
    private val marketPrice: Double,
) {
    fun error(price: Double?): AutocloseError? {
        if (price == null) return null
        if (price <= 0.0) return AutocloseError.InvalidAmount
        val mustBeAbove = when (type) {
            TpslType.TakeProfit -> direction == PerpetualDirection.Long
            TpslType.StopLoss -> direction == PerpetualDirection.Short
        }
        val onCorrectSide = if (mustBeAbove) price > marketPrice else price < marketPrice
        if (onCorrectSide) return null
        return if (mustBeAbove) AutocloseError.TriggerMustBeHigher else AutocloseError.TriggerMustBeLower
    }
}
