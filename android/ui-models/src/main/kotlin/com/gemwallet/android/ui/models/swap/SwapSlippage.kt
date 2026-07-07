package com.gemwallet.android.ui.models.swap

object SwapSlippage {
    val options: List<UInt> = listOf(30u, 50u, 100u, 300u, 500u)
    val defaultBps: UInt = 100u

    fun format(bps: UInt): String =
        bps.toLong().toBigDecimal().movePointLeft(2).stripTrailingZeros().toPlainString()

    fun label(bps: UInt): String = "${format(bps)}%"
}
