package com.gemwallet.android.data.coordinators.swap

import com.gemwallet.android.application.swap.coordinators.GetSwapAssets
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.FiatAssets
import uniffi.gemstone.GemAssetsService

class GetSwapAssetsImpl(
    private val assetsService: GemAssetsService,
) : GetSwapAssets {
    override suspend fun invoke(): FiatAssets = assetsService.getSwapAssets().decodeJson()
}
