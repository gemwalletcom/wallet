package com.gemwallet.android.domains.perpetual.autoclose

import com.gemwallet.android.domains.perpetual.toGem
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.TpslType
import uniffi.gemstone.AutocloseValidation as GemAutocloseValidation
import uniffi.gemstone.AutocloseValidator as GemAutocloseValidator

class AutocloseValidator(
    type: TpslType,
    direction: PerpetualDirection,
    marketPrice: Double,
) {
    private val validator = GemAutocloseValidator(type.toGem(), direction.toGem(), marketPrice)

    fun error(price: Double?): AutocloseError? {
        price ?: return null
        return when (validator.validate(price)) {
            GemAutocloseValidation.VALID -> null
            GemAutocloseValidation.INVALID_AMOUNT -> AutocloseError.InvalidAmount
            GemAutocloseValidation.TRIGGER_MUST_BE_HIGHER -> AutocloseError.TriggerMustBeHigher
            GemAutocloseValidation.TRIGGER_MUST_BE_LOWER -> AutocloseError.TriggerMustBeLower
        }
    }
}
