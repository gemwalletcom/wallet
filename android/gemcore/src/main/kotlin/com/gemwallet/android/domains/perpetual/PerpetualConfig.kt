package com.gemwallet.android.domains.perpetual

import uniffi.gemstone.Config

object PerpetualConfig {
    private val config get() = Config().getPerpetualConfig()

    val defaultLeverage: Int get() = config.defaultLeverage.toInt()

    val leverageOptions: List<Int> get() = config.leverageOptions.map { it.toInt() }

    fun selectLeverage(desired: Int, from: List<Int>): Int =
        Config().selectLeverage(
            desired.toUByte(),
            from.map { it.toByte() }.toByteArray(),
        ).toInt()
}
