package com.gemwallet.android.testkit

import uniffi.gemstone.FeeUnitType
import uniffi.gemstone.GemFeeRateRows
import java.math.BigInteger

fun mockGemFeeRateRows(selectedTotal: BigInteger = BigInteger("2"), unitDecimals: UInt = 0u) = GemFeeRateRows(
    rows = emptyList(),
    showsOptions = false,
    unitType = FeeUnitType.GWEI,
    unitDecimals = unitDecimals,
    supportsCustomFee = true,
    selectedTotal = selectedTotal,
    normalTotal = BigInteger("2"),
    customRate = null,
)
