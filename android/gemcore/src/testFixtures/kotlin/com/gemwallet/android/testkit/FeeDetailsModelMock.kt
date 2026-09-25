package com.gemwallet.android.testkit

import com.gemwallet.android.domains.confirm.FeeDetailsModel
import com.gemwallet.android.domains.confirm.FeeUIModel
import uniffi.gemstone.GemFeeRateRows

fun mockFeeDetailsModel(currentFee: FeeUIModel.FeeInfo = mockFeeInfo(), rows: GemFeeRateRows = mockGemFeeRateRows()): FeeDetailsModel = FeeDetailsModel(
    currentFee = currentFee,
    rows = rows,
)
