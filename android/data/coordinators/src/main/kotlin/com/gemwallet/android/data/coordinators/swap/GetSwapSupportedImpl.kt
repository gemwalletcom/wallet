package com.gemwallet.android.data.coordinators.swap

import com.gemwallet.android.application.swap.coordinators.GetSwapSupported
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.AssetId
import uniffi.gemstone.GemSwapper
import uniffi.gemstone.SwapperAssetList

class GetSwapSupportedImpl(
    private val gemSwapper: GemSwapper,
) : GetSwapSupported {
    override fun getSwapSupportChains(assetId: AssetId): SwapperAssetList {
        return gemSwapper.supportedChainsForFromAsset(assetId.toIdentifier())
    }
}
