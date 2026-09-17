package com.gemwallet.android.testkit

import uniffi.gemstone.FeeUnitType
import uniffi.gemstone.GemFeeRateRows
import java.math.BigInteger

fun mockGemFeeRateRows(
    selectedTotal: BigInteger = BigInteger("2"),
) = GemFeeRateRows(
    rows = emptyList(),
    unitType = FeeUnitType.GWEI,
    unitDecimals = 0u,
    supportsCustomFee = true,
    selectedTotal = selectedTotal,
    normalTotal = BigInteger("2"),
)
