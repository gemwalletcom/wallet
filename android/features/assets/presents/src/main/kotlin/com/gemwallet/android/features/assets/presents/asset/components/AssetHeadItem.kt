package com.gemwallet.android.features.assets.presents.asset.components

import androidx.compose.runtime.Composable
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.features.assets.viewmodels.asset.models.AssetAction
import com.gemwallet.android.ui.components.list_head.AssetHeadActions
import com.gemwallet.android.ui.components.list_head.ValueListHead
import uniffi.gemstone.GemHeaderButtonAction
import uniffi.gemstone.GemValueHeader

@Composable
internal fun AssetHeadItem(header: GemValueHeader, onAction: (AssetAction) -> Unit) {
    ValueListHead(header = header) {
        AssetHeadActions(header.actions ?: return@ValueListHead) { action ->
            when (action) {
                is GemHeaderButtonAction.Send -> action.assetId?.toAssetId()?.let { onAction(AssetAction.Transfer(it)) }
                is GemHeaderButtonAction.Receive -> action.assetId?.toAssetId()?.let { onAction(AssetAction.Receive(it)) }
                is GemHeaderButtonAction.Buy -> action.assetId?.toAssetId()?.let { onAction(AssetAction.Buy(it)) }
                is GemHeaderButtonAction.Swap -> action.payAssetId?.toAssetId()?.let { onAction(AssetAction.Swap(fromAssetId = it, toAssetId = action.receiveAssetId?.toAssetId())) }
                is GemHeaderButtonAction.Deposit, is GemHeaderButtonAction.Withdraw, GemHeaderButtonAction.SendCollectible, GemHeaderButtonAction.CollectibleMenu -> Unit
            }
        }
    }
}
