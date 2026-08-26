package com.gemwallet.android.data.coordinators.fiat

import com.gemwallet.android.application.fiat.coordinators.GetBuyableFiatAssets
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.FiatAssets
import com.wallet.core.primitives.FiatQuoteType
import uniffi.gemstone.GemAssetsService

class GetBuyableFiatAssetsImpl(
    private val assetsService: GemAssetsService,
) : GetBuyableFiatAssets {
    override suspend fun invoke(): FiatAssets =
        assetsService.getFiatAssets(FiatQuoteType.Buy.toJson()).decodeJson()
}
