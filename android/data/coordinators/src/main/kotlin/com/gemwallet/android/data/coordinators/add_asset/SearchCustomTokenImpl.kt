package com.gemwallet.android.data.coordinators.add_asset

import android.util.Log
import com.gemwallet.android.application.add_asset.cases.SearchCustomToken
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.AssetId
import uniffi.gemstone.GemAssetsService

class SearchCustomTokenImpl(
    private val assetsService: GemAssetsService,
) : SearchCustomToken {

    override suspend fun invoke(assetId: AssetId): Boolean =
        runCatchingCancellable { assetsService.ensureTokenAsset(assetId.toIdentifier()) }
            .onFailure { Log.e(TAG, "token asset lookup failed", it) }
            .isSuccess

    private companion object {
        const val TAG = "SearchCustomToken"
    }
}
