package com.gemwallet.android.ext

import com.wallet.core.primitives.FeeUnitType
import uniffi.gemstone.GemGasPriceType
import java.math.BigInteger

fun GemGasPriceType.totalFee(): BigInteger = when (this) {
    is GemGasPriceType.Regular -> gasPrice.toBigInteger()
    is GemGasPriceType.Eip1559 -> gasPrice.toBigInteger() + priorityFee.toBigInteger()
    is GemGasPriceType.Solana -> gasPrice.toBigInteger() + priorityFee.toBigInteger()
}

val FeeUnitType.gasPriceDecimals: Int?
    get() = when (this) {
        FeeUnitType.SatVb -> 0
        FeeUnitType.Gwei -> 9
        FeeUnitType.Native -> null
    }

val FeeUnitType.gasPriceSymbol: String?
    get() = when (this) {
        FeeUnitType.SatVb, FeeUnitType.Gwei -> string
        FeeUnitType.Native -> null
    }
