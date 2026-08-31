package com.gemwallet.android.features.asset.presents.details.components

import androidx.compose.foundation.lazy.LazyListScope
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.ext.hasNativeAsset
import com.gemwallet.android.ext.isTokenSupported
import com.gemwallet.android.ext.type
import com.gemwallet.android.features.asset.presents.details.AssetDetailsAction
import com.gemwallet.android.features.asset.viewmodels.details.models.AssetInfoUIModel
import com.gemwallet.android.ui.components.list_item.property.PropertyNetworkItem
import com.gemwallet.android.ui.models.ListPosition
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetSubtype

internal fun LazyListScope.network(
    uiState: AssetInfoUIModel,
    onAction: (AssetDetailsAction) -> Unit,
) {
    val networkNavigationAction = uiState.asset.networkNavigationAction()
    item {
        PropertyNetworkItem(
            asset = uiState.asset,
            onOpenNetwork = networkNavigationAction?.let { { onAction(it) } },
            listPosition = ListPosition.Last,
        )
    }
}

internal fun Asset.networkNavigationAction(
    hasNativeAsset: Boolean = chain.hasNativeAsset(),
): AssetDetailsAction.Navigation? {
    val networkAssetId = AssetId(chain)
    val networkAssetsAction = if (chain.isTokenSupported()) AssetDetailsAction.OpenNetworkAssets(chain) else null
    return when (id.type()) {
        AssetSubtype.NATIVE -> networkAssetsAction
        AssetSubtype.TOKEN -> if (hasNativeAsset) AssetDetailsAction.OpenNetwork(networkAssetId) else networkAssetsAction
    }
}
