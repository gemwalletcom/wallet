package com.gemwallet.android.data.coordinators.fiat

import com.gemwallet.android.application.fiat.coordinators.GetBuyableFiatAssets
import com.gemwallet.android.data.services.gemapi.GemDeviceApiClient
import com.wallet.core.primitives.FiatAssets

class GetBuyableFiatAssetsImpl(
    private val gemDeviceApiClient: GemDeviceApiClient,
) : GetBuyableFiatAssets {
    override suspend fun invoke(): FiatAssets {
        return gemDeviceApiClient.getBuyableFiatAssets()
    }
}
