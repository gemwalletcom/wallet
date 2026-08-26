package com.gemwallet.android.data.coordinators.fiat

import com.gemwallet.android.application.fiat.coordinators.GetSellableFiatAssets
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.FiatAssets
import com.wallet.core.primitives.FiatQuoteType
import uniffi.gemstone.GemAssetsService

class GetSellableFiatAssetsImpl(
    private val assetsService: GemAssetsService,
) : GetSellableFiatAssets {
    override suspend fun invoke(): FiatAssets =
        assetsService.getFiatAssets(FiatQuoteType.Sell.toJson()).decodeJson()
}
