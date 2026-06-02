package com.gemwallet.android.domains.perpetual

import uniffi.gemstone.Config

object PerpetualConfig {
    private val config get() = Config().getPerpetualConfig()

    val defaultLeverage: Int get() = config.defaultLeverage.toInt()

    val leverageOptions: List<Int> get() = config.leverageOptions.toUnsignedInts()

    val takeProfitOptions: List<Int> get() = config.takeProfitPercentOptions.toUnsignedInts()

    val stopLossOptions: List<Int> get() = config.stopLossPercentOptions.toUnsignedInts()

    val defaultTakeProfit: Int get() = config.defaultTakeProfitPercent.toInt()

    val defaultStopLoss: Int get() = config.defaultStopLossPercent.toInt()

    fun autocloseSuggestions(leverage: Int): List<Int> =
        Config().getAutocloseSuggestions(leverage.toUByte()).toUnsignedInts()

    fun selectLeverage(desired: Int, from: List<Int>): Int =
        Config().selectLeverage(
            desired.toUByte(),
            from.map { it.toByte() }.toByteArray(),
        ).toInt()
}

private fun ByteArray.toUnsignedInts(): List<Int> = map { it.toUByte().toInt() }
