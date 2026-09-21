package com.gemwallet.android.features.asset.viewmodels.details.models

import com.gemwallet.android.ext.requireChain
import com.gemwallet.android.ext.toPrimitives
import com.wallet.core.primitives.AssetId
import uniffi.gemstone.GemAssetNetworkDestination
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemListRowTitle

fun GemAssetNetworkDestination?.navigation(): AssetDetailsAction.Navigation? = when (this) {
    is GemAssetNetworkDestination.Asset -> AssetDetailsAction.OpenNetwork(asset.toPrimitives().id)
    is GemAssetNetworkDestination.Assets -> AssetDetailsAction.OpenNetworkAssets(chain.requireChain())
    null -> null
}

fun GemListRow.detailsAction(assetId: AssetId): AssetDetailsAction? = when (this) {
    is GemListRow.Link -> when (title) {
        GemListRowTitle.PIN, GemListRowTitle.UNPIN -> AssetDetailsAction.Pin
        GemListRowTitle.ADD_TO_WALLET -> AssetDetailsAction.Add
        GemListRowTitle.PRICE_ALERTS -> AssetDetailsAction.OpenPriceAlerts(assetId)
        else -> null
    }

    else -> null
}
