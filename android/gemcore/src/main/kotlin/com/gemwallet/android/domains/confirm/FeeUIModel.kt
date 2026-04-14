package com.gemwallet.android.domains.confirm

import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.FeePriority
import java.math.BigInteger

sealed interface FeeUIModel {
    data object Calculating : FeeUIModel
    data object Error : FeeUIModel
    class FeeInfo(
        val amount: BigInteger,
        val cryptoAmount: String,
        val fiatAmount: String,
        val feeAsset: Asset,
        val priority: FeePriority,
    ) : FeeUIModel
}
