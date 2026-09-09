package com.gemwallet.android.domains.price

import uniffi.gemstone.PriceAlertDirection

enum class ValueDirection {
    None,
    Up,
    Down,
}

fun Double?.toValueDirection(): ValueDirection = when {
    this == null || !isFinite() -> ValueDirection.None
    this > 0.0 -> ValueDirection.Up
    this < 0.0 -> ValueDirection.Down
    else -> ValueDirection.None
}

fun PriceAlertDirection?.toValueDirection(): ValueDirection = when (this) {
    PriceAlertDirection.UP -> ValueDirection.Up
    PriceAlertDirection.DOWN -> ValueDirection.Down
    null -> ValueDirection.None
}
