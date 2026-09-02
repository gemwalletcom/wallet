package com.gemwallet.android.domains.perpetual.autoclose

import com.wallet.core.primitives.TpslType
import uniffi.gemstone.AutocloseValidation
import uniffi.gemstone.GemAutocloseField

data class AutocloseField(
    val type: TpslType,
    val price: Double?,
    val originalPrice: Double?,
    val formattedPrice: String?,
    val validation: AutocloseValidation,
    val orderId: ULong?,
) {
    fun toGem(): GemAutocloseField = GemAutocloseField(
        price = price,
        originalPrice = originalPrice,
        formattedPrice = formattedPrice,
        isValid = price != null && validation == AutocloseValidation.VALID,
        orderId = orderId,
    )
}
