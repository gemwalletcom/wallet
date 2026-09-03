package com.gemwallet.android.domains.perpetual.autoclose

import com.gemwallet.android.domains.perpetual.toGem
import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.TpslType
import uniffi.gemstone.AutocloseValidation
import uniffi.gemstone.AutocloseValidator as GemAutocloseValidator

class AutocloseValidator(
    type: TpslType,
    direction: PerpetualDirection,
    marketPrice: Double,
) {
    private val validator = GemAutocloseValidator(type.toGem(), direction.toGem(), marketPrice)

    fun validate(price: Double?): AutocloseValidation =
        price?.let { validator.validate(it) } ?: AutocloseValidation.VALID
}
