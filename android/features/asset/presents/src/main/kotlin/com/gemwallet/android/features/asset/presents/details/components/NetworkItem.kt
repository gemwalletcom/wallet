package com.gemwallet.android.features.asset.presents.details.components

import androidx.compose.foundation.lazy.LazyListScope
import com.gemwallet.android.features.asset.presents.details.AssetDetailsAction
import com.gemwallet.android.ext.requireChain
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.asset.viewmodels.details.models.AssetInfoUIModel
import uniffi.gemstone.GemAssetNetworkDestination
import com.gemwallet.android.ui.components.list_item.property.PropertyNetworkItem
import com.gemwallet.android.ui.models.ListPosition

internal fun LazyListScope.network(
    uiState: AssetInfoUIModel,
    onAction: (AssetDetailsAction) -> Unit,
) {
    val networkNavigationAction = uiState.networkNavigation
    item {
        PropertyNetworkItem(
            asset = uiState.asset,
            onOpenNetwork = networkNavigationAction?.let { { onAction(it) } },
            listPosition = ListPosition.Last,
        )
    }
}

private val AssetInfoUIModel.networkNavigation: AssetDetailsAction.Navigation?
    get() = when (val destination = networkDestination) {
        is GemAssetNetworkDestination.Asset -> AssetDetailsAction.OpenNetwork(destination.asset.toPrimitives().id)
        is GemAssetNetworkDestination.Assets -> AssetDetailsAction.OpenNetworkAssets(destination.chain.requireChain())
        null -> null
    }
