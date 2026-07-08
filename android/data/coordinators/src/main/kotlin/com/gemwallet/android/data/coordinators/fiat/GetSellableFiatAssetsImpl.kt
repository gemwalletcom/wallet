package com.gemwallet.android.data.coordinators.fiat

import com.gemwallet.android.application.fiat.coordinators.GetSellableFiatAssets
import com.gemwallet.android.data.services.gemapi.GemApiClient
import com.wallet.core.primitives.FiatAssets
import com.wallet.core.primitives.FiatQuoteType

class GetSellableFiatAssetsImpl(
    private val gemApiClient: GemApiClient,
) : GetSellableFiatAssets {
    override suspend fun invoke(): FiatAssets {
        return gemApiClient.getFiatAssets(FiatQuoteType.Sell.string)
    }
}
