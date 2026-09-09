package com.gemwallet.android

import android.util.Log
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.ui.navigation.routes.AssetRoute
import com.gemwallet.android.ui.navigation.routes.FiatInputRoute
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.FiatQuoteType
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemAssetsService
import javax.inject.Inject

class AssetNavigation @Inject constructor(
    private val assetsService: GemAssetsService,
) {
    suspend fun assetRoute(assetId: AssetId?): AssetRoute? = openAsset(assetId)?.let { AssetRoute(it.id) }

    suspend fun fiatRoute(assetId: AssetId?, amount: Int?, type: FiatQuoteType): FiatInputRoute? {
        return openAsset(assetId)?.let { FiatInputRoute(it.id, amount, type) }
    }

    private suspend fun openAsset(assetId: AssetId?): Asset? {
        if (assetId == null) {
            return null
        }
        return withContext(Dispatchers.IO) {
            runCatchingCancellable { assetsService.openAsset(assetId.toIdentifier()) }
                .onFailure { Log.e(TAG, "opening ${assetId.toIdentifier()} failed", it) }
                .getOrNull()
        }?.toPrimitives()
    }

    private companion object {
        const val TAG = "AssetNavigation"
    }
}
