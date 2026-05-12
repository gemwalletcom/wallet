package com.gemwallet.android.domains.price

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
