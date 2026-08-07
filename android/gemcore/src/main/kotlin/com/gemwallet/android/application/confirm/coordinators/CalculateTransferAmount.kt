package com.gemwallet.android.application.confirm.coordinators

import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.ConfirmParams
import java.math.BigInteger

interface CalculateTransferAmount {
    operator fun invoke(
        params: ConfirmParams,
        availableValue: BigInteger,
        feeAssetInfo: AssetInfo,
        fee: BigInteger,
    ): BigInteger
}
