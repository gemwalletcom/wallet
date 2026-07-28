package com.gemwallet.android.features.asset.presents.details.components

import androidx.compose.foundation.lazy.LazyListScope
import com.gemwallet.android.ext.assetType
import com.gemwallet.android.features.asset.presents.details.AssetDetailsAction
import com.gemwallet.android.features.asset.viewmodels.details.models.AssetInfoUIModel
import com.gemwallet.android.ui.components.list_item.property.PropertyNetworkItem
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.actions.AssetIdAction
import com.wallet.core.primitives.AssetType

internal fun LazyListScope.network(
    uiState: AssetInfoUIModel,
    onAction: (AssetDetailsAction) -> Unit,
) {
    val chain = uiState.asset.id.chain
    val onOpenNetwork: AssetIdAction? = when {
        uiState.tokenType != AssetType.NATIVE -> AssetIdAction { onAction(AssetDetailsAction.OpenNetwork(it)) }
        chain.assetType() != null -> AssetIdAction { onAction(AssetDetailsAction.OpenNetworkAssets(chain)) }
        else -> null
    }
    item {
        PropertyNetworkItem(
            asset = uiState.asset,
            onOpenNetwork = onOpenNetwork,
            listPosition = ListPosition.Last,
        )
    }
}
