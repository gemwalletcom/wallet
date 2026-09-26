package com.gemwallet.android.features.assets.viewmodels.details.models

import com.gemwallet.android.ext.requireChain
import com.gemwallet.android.ext.toPrimitives
import com.wallet.core.primitives.AssetId
import uniffi.gemstone.GemAssetNetworkDestination
import uniffi.gemstone.GemBalanceRow
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

    is GemListRow.Quote -> when (title) {
        GemListRowTitle.PRICE -> AssetDetailsAction.OpenChart(assetId)
        else -> null
    }

    is GemListRow.Amount -> when (title) {
        GemListRowTitle.STAKE_APR -> AssetDetailsAction.Earn(assetId)
        else -> null
    }

    is GemListRow.Text -> when (title) {
        GemListRowTitle.STAKE_APR -> AssetDetailsAction.Earn(assetId)
        else -> null
    }

    else -> null
}

fun GemBalanceRow.detailsAction(assetId: AssetId): AssetDetailsAction? = when (this) {
    is GemBalanceRow.Available, is GemBalanceRow.PendingUnconfirmed -> null
    is GemBalanceRow.Staked -> AssetDetailsAction.Stake(assetId)
    is GemBalanceRow.Earn -> AssetDetailsAction.Earn(assetId)
    is GemBalanceRow.Reserved -> url?.let { AssetDetailsAction.OpenUrl(it) }
}

internal fun GemListRow.networkAction(network: AssetDetailsAction.Navigation?): AssetDetailsAction? = when (this) {
    is GemListRow.Network -> network
    else -> null
}
