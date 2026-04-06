package com.gemwallet.android.features.asset.presents.details.components

import androidx.compose.foundation.lazy.LazyListScope
import com.gemwallet.android.ui.components.list_item.property.PropertyNetworkItem
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.actions.AssetIdAction
import com.gemwallet.android.features.asset.viewmodels.details.models.AssetInfoUIModel
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetType

internal data class NetworkRowState(
    val asset: Asset,
    val onOpenNetwork: AssetIdAction?,
)

internal fun AssetInfoUIModel.networkRowState(openNetwork: AssetIdAction) = NetworkRowState(
    asset = asset,
    onOpenNetwork = openNetwork.takeIf { tokenType != AssetType.NATIVE },
)

internal fun LazyListScope.network(
    uiState: AssetInfoUIModel,
    openNetwork: AssetIdAction,
) {
    val rowState = uiState.networkRowState(openNetwork)
    item {
        PropertyNetworkItem(
            asset = rowState.asset,
            onOpenNetwork = rowState.onOpenNetwork,
            listPosition = ListPosition.Last
        )
    }
}
