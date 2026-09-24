package com.gemwallet.android.domains.price

import uniffi.gemstone.GemValueTone
import uniffi.gemstone.PriceAlertDirection

fun PriceAlertDirection?.tone(): GemValueTone = when (this) {
    PriceAlertDirection.UP -> GemValueTone.POSITIVE
    PriceAlertDirection.DOWN -> GemValueTone.NEGATIVE
    null -> GemValueTone.NEUTRAL
}
