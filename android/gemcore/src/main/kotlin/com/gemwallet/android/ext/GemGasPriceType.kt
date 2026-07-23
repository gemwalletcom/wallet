package com.gemwallet.android.ext

import com.wallet.core.primitives.FeeUnitType
import uniffi.gemstone.FeeConfig
import uniffi.gemstone.GemGasPriceType
import java.math.BigInteger

fun GemGasPriceType.totalFee(): BigInteger = when (this) {
    is GemGasPriceType.Regular -> gasPrice.toBigInteger()
    is GemGasPriceType.Eip1559 -> gasPrice.toBigInteger() + priorityFee.toBigInteger()
    is GemGasPriceType.Solana -> gasPrice.toBigInteger() + priorityFee.toBigInteger()
}

fun feeRateDecimals(feeUnitType: FeeUnitType?, feeConfig: FeeConfig, assetDecimals: Int): Int =
    when (feeUnitType) {
        FeeUnitType.SatVb, FeeUnitType.Gwei -> feeConfig.decimals.toInt()
        FeeUnitType.Native, null -> assetDecimals
    }
