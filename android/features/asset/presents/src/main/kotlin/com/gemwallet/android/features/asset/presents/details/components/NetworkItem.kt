package com.gemwallet.android.features.asset.presents.details.components

import androidx.compose.foundation.lazy.LazyListScope
import com.gemwallet.android.features.asset.presents.details.AssetDetailsAction
import com.gemwallet.android.features.asset.viewmodels.details.models.AssetInfoUIModel
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
        is AssetInfoUIModel.NetworkDestination.Asset -> AssetDetailsAction.OpenNetwork(destination.assetId)
        is AssetInfoUIModel.NetworkDestination.Assets -> AssetDetailsAction.OpenNetworkAssets(destination.chain)
        null -> null
    }
