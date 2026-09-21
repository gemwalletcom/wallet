package com.gemwallet.android.domains.price

import uniffi.gemstone.GemValueTone
import uniffi.gemstone.PriceAlertDirection
import uniffi.gemstone.valueTone

fun Double?.tone(): GemValueTone = this?.takeIf { it.isFinite() }?.let(::valueTone) ?: GemValueTone.NEUTRAL

fun PriceAlertDirection?.tone(): GemValueTone = when (this) {
    PriceAlertDirection.UP -> GemValueTone.POSITIVE
    PriceAlertDirection.DOWN -> GemValueTone.NEGATIVE
    null -> GemValueTone.NEUTRAL
}
