package com.gemwallet.android.application.swap.coordinators

import com.wallet.core.primitives.AssetId
import uniffi.gemstone.SwapperAssetList

interface GetSwapSupported {
    fun getSwapSupportChains(assetId: AssetId): SwapperAssetList
}
