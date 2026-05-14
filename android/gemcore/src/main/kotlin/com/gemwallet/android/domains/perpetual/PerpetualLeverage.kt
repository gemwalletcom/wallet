package com.gemwallet.android.domains.perpetual

const val perpetualLeverageDefault: Int = 5

val perpetualLeverageOptions: List<Int> = listOf(1, 2, 3, 5, 10, 20, 25, 30, 40, 50)

fun Int.formatLeverage(): String = "${this}x"

fun getLeverage(desired: Int, from: List<Int>): Int =
    from.filter { it <= desired }.maxOrNull() ?: from.minOrNull() ?: perpetualLeverageOptions.first()
