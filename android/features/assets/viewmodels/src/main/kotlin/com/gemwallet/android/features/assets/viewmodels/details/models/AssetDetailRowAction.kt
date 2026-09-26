package com.gemwallet.android.features.assets.viewmodels.details.models

import com.gemwallet.android.ext.requireChain
import com.gemwallet.android.ext.toPrimitives
import com.wallet.core.primitives.AssetId
import uniffi.gemstone.GemAssetNetworkDestination
import uniffi.gemstone.GemRowTap

fun GemAssetNetworkDestination?.navigation(): AssetDetailsAction.Navigation? = when (this) {
    is GemAssetNetworkDestination.Asset -> AssetDetailsAction.OpenNetwork(asset.toPrimitives().id)
    is GemAssetNetworkDestination.Assets -> AssetDetailsAction.OpenNetworkAssets(chain.requireChain())
    null -> null
}

fun GemRowTap.detailsAction(assetId: AssetId, network: AssetDetailsAction.Navigation?): AssetDetailsAction? = when (this) {
    GemRowTap.Price -> AssetDetailsAction.OpenChart(assetId)
    GemRowTap.Network -> network
    GemRowTap.Earn -> AssetDetailsAction.Earn(assetId)
    GemRowTap.Stake -> AssetDetailsAction.Stake(assetId)
    GemRowTap.PriceAlerts -> AssetDetailsAction.OpenPriceAlerts(assetId)
    GemRowTap.Pin -> AssetDetailsAction.Pin
    GemRowTap.AddToWallet -> AssetDetailsAction.Add
    is GemRowTap.Explorer -> AssetDetailsAction.OpenUrl(url)
    else -> null
}
