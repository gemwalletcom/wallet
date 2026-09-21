package com.gemwallet.android.testkit

import uniffi.gemstone.GemAmountInput
import java.math.BigInteger

fun mockGemAmountInput(available: BigInteger = BigInteger.ZERO, max: BigInteger = available, reservedFee: BigInteger? = null) = GemAmountInput(
    availableValue = available,
    maxValue = max,
    reservedFee = reservedFee,
    canChangeValue = true,
    showsAssetBalance = true,
    usesWholeAmounts = false,
)
