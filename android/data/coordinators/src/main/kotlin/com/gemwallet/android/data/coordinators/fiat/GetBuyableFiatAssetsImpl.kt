package com.gemwallet.android.data.coordinators.fiat

import com.gemwallet.android.application.fiat.coordinators.GetBuyableFiatAssets
import com.gemwallet.android.data.services.gemapi.GemApiClient
import com.wallet.core.primitives.FiatAssets
import com.wallet.core.primitives.FiatQuoteType

class GetBuyableFiatAssetsImpl(
    private val gemApiClient: GemApiClient,
) : GetBuyableFiatAssets {
    override suspend fun invoke(): FiatAssets {
        return gemApiClient.getFiatAssets(FiatQuoteType.Buy.string)
    }
}
