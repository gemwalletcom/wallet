package com.gemwallet.android.testkit

import com.gemwallet.android.domains.confirm.FeeAssetUIModel
import com.gemwallet.android.domains.confirm.FeeDetailsModel
import com.gemwallet.android.domains.confirm.FeeUIModel
import uniffi.gemstone.GemFeeRateRows
import java.math.BigInteger

fun mockFeeDetailsModel(currentFee: FeeUIModel.FeeInfo = mockFeeInfo(), rows: GemFeeRateRows = mockGemFeeRateRows()): FeeDetailsModel = FeeDetailsModel(
    currentFee = currentFee,
    feeAsset = FeeAssetUIModel(asset = currentFee.feeAsset, price = null, available = BigInteger("1000000")),
    rows = rows,
)
