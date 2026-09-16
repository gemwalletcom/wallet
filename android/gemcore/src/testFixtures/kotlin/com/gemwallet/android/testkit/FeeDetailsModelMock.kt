package com.gemwallet.android.testkit

import com.gemwallet.android.domains.confirm.FeeAssetUIModel
import com.gemwallet.android.domains.confirm.FeeDetailsModel
import java.math.BigInteger

fun mockFeeDetailsModel(): FeeDetailsModel {
    val currentFee = mockFeeInfo()
    return FeeDetailsModel(
        currentFee = currentFee,
        feeAsset = FeeAssetUIModel(asset = currentFee.feeAsset, price = null, available = BigInteger("1000000")),
        rows = mockGemFeeRateRows(),
    )
}
