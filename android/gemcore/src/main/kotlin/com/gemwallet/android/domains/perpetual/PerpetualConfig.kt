package com.gemwallet.android.domains.perpetual

import com.gemwallet.android.ext.toAssetId
import com.wallet.core.primitives.AssetId
import uniffi.gemstone.Config
import java.math.BigInteger

object PerpetualConfig {
    private val config get() = Config().getPerpetualConfig()

    val defaultLeverage: Int get() = config.defaultLeverage.toInt()

    val depositAddress: String get() = config.depositAddress

    val depositAssetId: AssetId get() = requireNotNull(config.depositAssetId.toAssetId()) {
        "Invalid perpetual deposit asset id: ${config.depositAssetId}"
    }

    val minDeposit: BigInteger get() = config.minDeposit.toLong().toBigInteger()

    val minWithdraw: BigInteger get() = config.minWithdraw.toLong().toBigInteger()

    val pricesUpdateIntervalSeconds: Int get() = config.pricesUpdateIntervalSeconds.toInt()

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
