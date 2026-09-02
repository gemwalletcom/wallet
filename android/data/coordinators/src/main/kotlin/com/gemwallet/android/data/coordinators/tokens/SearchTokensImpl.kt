package com.gemwallet.android.data.coordinators.tokens

import android.util.Log
import com.gemwallet.android.application.tokens.cases.SearchTokens
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Currency
import uniffi.gemstone.GemAssetsService

class SearchTokensImpl(
    private val assetsService: GemAssetsService,
) : SearchTokens {

    override suspend fun search(assetIds: List<AssetId>, currency: Currency): Boolean =
        runCatchingCancellable { assetsService.syncAssets(assetIds.map { it.toIdentifier() }, currency.toGem()) }
            .onFailure { Log.e(TAG, "assets sync failed", it) }
            .isSuccess

    private companion object {
        const val TAG = "SearchTokens"
    }
}
