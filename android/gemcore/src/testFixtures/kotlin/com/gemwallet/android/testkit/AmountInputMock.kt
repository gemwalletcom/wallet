package com.gemwallet.android.testkit

import uniffi.gemstone.GemAmountInput
import uniffi.gemstone.GemFormattedNumber
import uniffi.gemstone.GemValueStyle
import uniffi.gemstone.formattedAmount
import java.math.BigInteger

fun mockGemAmountInput(
    available: BigInteger = BigInteger.ZERO,
    max: BigInteger = available,
    reservedFee: BigInteger? = null,
    balance: GemFormattedNumber = formattedAmount(0.0, null, GemValueStyle.AUTO),
) = GemAmountInput(
    availableValue = available,
    balance = balance,
    maxValue = max,
    reservedFee = reservedFee,
    canChangeValue = true,
    showsAssetBalance = true,
    usesWholeAmounts = false,
)
