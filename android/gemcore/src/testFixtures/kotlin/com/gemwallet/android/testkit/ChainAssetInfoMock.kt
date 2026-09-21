package com.gemwallet.android.testkit

import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.ChainAssetInfo

fun mockChainAssetInfo(assetInfo: AssetInfo = mockAssetInfo()) = ChainAssetInfo(
    assetInfo = assetInfo,
    feeAssetInfo = assetInfo,
)
