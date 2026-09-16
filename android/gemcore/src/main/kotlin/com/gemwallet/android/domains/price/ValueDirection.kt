package com.gemwallet.android.domains.price

import uniffi.gemstone.GemValueTone
import uniffi.gemstone.PriceAlertDirection
import uniffi.gemstone.valueTone

enum class ValueDirection {
    None,
    Up,
    Down,
}

fun Double?.toValueDirection(): ValueDirection = when (this?.takeIf { it.isFinite() }?.let(::valueTone)) {
    GemValueTone.POSITIVE -> ValueDirection.Up
    GemValueTone.NEGATIVE -> ValueDirection.Down
    GemValueTone.NEUTRAL, GemValueTone.PLAIN, null -> ValueDirection.None
}

fun PriceAlertDirection?.toValueDirection(): ValueDirection = when (this) {
    PriceAlertDirection.UP -> ValueDirection.Up
    PriceAlertDirection.DOWN -> ValueDirection.Down
    null -> ValueDirection.None
}
