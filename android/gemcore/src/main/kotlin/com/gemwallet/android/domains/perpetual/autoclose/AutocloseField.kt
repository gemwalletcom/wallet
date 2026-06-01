package com.gemwallet.android.domains.perpetual.autoclose

import com.wallet.core.primitives.TpslType

data class AutocloseField(
    val type: TpslType,
    val price: Double?,
    val originalPrice: Double?,
    val formattedPrice: String?,
    val error: AutocloseError?,
    val orderId: ULong?,
) {
    val isValid: Boolean get() = price != null && error == null
    val hasChanged: Boolean get() = price != originalPrice
    val isCleared: Boolean get() = price == null && originalPrice != null
    val hasExisting: Boolean get() = originalPrice != null
    val shouldSet: Boolean get() = isValid && hasChanged
    val shouldUpdate: Boolean get() = shouldSet || isCleared
    val shouldCancel: Boolean get() = isCleared || (shouldSet && hasExisting)
    val hasPendingChange: Boolean get() = isCleared || (price != null && hasChanged)
}
