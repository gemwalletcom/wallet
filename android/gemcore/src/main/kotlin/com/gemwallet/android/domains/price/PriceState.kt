package com.gemwallet.android.domains.price

enum class PriceState {
    None,
    Up,
    Down,
}

fun Double?.toPriceState(): PriceState = when {
    this == null || !isFinite() -> PriceState.None
    this > 0.0 -> PriceState.Up
    this < 0.0 -> PriceState.Down
    else -> PriceState.None
}
