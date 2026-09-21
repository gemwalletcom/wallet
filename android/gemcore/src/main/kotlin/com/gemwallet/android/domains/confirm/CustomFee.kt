package com.gemwallet.android.domains.confirm

import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.math.numberFormat
import com.gemwallet.android.model.text
import uniffi.gemstone.GemCustomFee
import uniffi.gemstone.GemCustomFeeCheck
import uniffi.gemstone.GemFeeRateRows
import java.math.BigInteger

data class CustomFee(val rate: BigInteger?, val placeholder: String, val networkFee: FeeUIModel.FeeInfo, val check: GemCustomFeeCheck, val isConfirmEnabled: Boolean) {
    companion object {
        fun from(input: String, currentFee: FeeUIModel.FeeInfo, rows: GemFeeRateRows): CustomFee = GemCustomFee.estimate(
            chain = currentFee.feeAsset.chain.string,
            input = input,
            format = numberFormat(),
            rows = rows,
            loadedFee = currentFee.amount,
        ).use { estimate ->
            CustomFee(
                rate = estimate.rate(),
                placeholder = estimate.placeholder()?.text() ?: "",
                networkFee = FeeUIModel.FeeInfo(estimate.feeValue(), currentFee.feeAsset, currentFee.price, currentFee.currency, currentFee.priority),
                check = estimate.check(),
                isConfirmEnabled = estimate.isValid(),
            )
        }
    }
}
