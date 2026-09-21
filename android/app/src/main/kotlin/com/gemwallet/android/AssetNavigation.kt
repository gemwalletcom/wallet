package com.gemwallet.android

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
import uniffi.gemstone.GemAssetsServiceInterface
import javax.inject.Inject

class AssetNavigation @Inject constructor(private val assetsService: GemAssetsServiceInterface) {
    suspend fun assetRoute(assetId: AssetId?): AssetRoute? = openAsset(assetId)?.let { AssetRoute(it.id) }

    suspend fun fiatRoute(assetId: AssetId?, amount: Int?, type: FiatQuoteType): FiatInputRoute? = openAsset(assetId)?.let { FiatInputRoute(it.id, amount, type) }

    private suspend fun openAsset(assetId: AssetId?): Asset? {
        if (assetId == null) {
            return null
        }
        return withContext(Dispatchers.IO) { assetsService.openAsset(assetId.toIdentifier()) }?.toPrimitives()
    }
}
