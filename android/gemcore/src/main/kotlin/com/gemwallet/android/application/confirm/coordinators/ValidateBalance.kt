package com.gemwallet.android.application.confirm.coordinators

import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.SignerParams
import java.math.BigInteger

interface ValidateBalance {
    operator fun invoke(
        signerParams: SignerParams,
        assetInfo: AssetInfo,
        feeAssetInfo: AssetInfo,
        assetBalance: BigInteger,
    )
}
