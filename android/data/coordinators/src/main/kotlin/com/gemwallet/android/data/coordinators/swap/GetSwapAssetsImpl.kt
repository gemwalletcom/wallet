package com.gemwallet.android.data.coordinators.swap

import com.gemwallet.android.application.swap.coordinators.GetSwapAssets
import com.gemwallet.android.data.services.gemapi.GemApiClient
import com.wallet.core.primitives.FiatAssets

class GetSwapAssetsImpl(
    private val gemApiClient: GemApiClient,
) : GetSwapAssets {
    override suspend fun invoke(): FiatAssets {
        return gemApiClient.getSwapAssets()
    }
}
