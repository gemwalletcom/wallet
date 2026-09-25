package com.gemwallet.android.testkit

import com.gemwallet.android.domains.confirm.FeeDetailsModel
import com.gemwallet.android.domains.confirm.FeeUIModel
import uniffi.gemstone.FeeUnitType
import uniffi.gemstone.GemFeeRateRows
import java.math.BigInteger

fun mockFeeDetailsModel(
    currentFee: FeeUIModel.FeeInfo = mockFeeInfo(),
    rows: GemFeeRateRows = mockGemFeeRateRows(unitType = FeeUnitType.GWEI, unitDecimals = 0u, supportsCustomFee = true, selectedTotal = BigInteger("2"), normalTotal = BigInteger("2")),
): FeeDetailsModel = FeeDetailsModel(
    currentFee = currentFee,
    rows = rows,
)
