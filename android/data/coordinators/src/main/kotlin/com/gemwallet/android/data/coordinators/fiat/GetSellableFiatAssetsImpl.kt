package com.gemwallet.android.data.coordinators.fiat

import com.gemwallet.android.application.fiat.coordinators.GetSellableFiatAssets
import com.gemwallet.android.data.services.gemapi.GemDeviceApiClient
import com.wallet.core.primitives.FiatAssets

class GetSellableFiatAssetsImpl(
    private val gemDeviceApiClient: GemDeviceApiClient,
) : GetSellableFiatAssets {
    override suspend fun invoke(): FiatAssets {
        return gemDeviceApiClient.getSellableFiatAssets()
    }
}
