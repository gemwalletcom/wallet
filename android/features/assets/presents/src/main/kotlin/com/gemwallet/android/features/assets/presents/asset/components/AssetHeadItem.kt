package com.gemwallet.android.features.assets.presents.asset.components

import androidx.compose.runtime.Composable
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.features.assets.viewmodels.asset.models.AssetAction
import com.gemwallet.android.ui.components.list_head.AssetHeadActions
import com.gemwallet.android.ui.components.list_head.ValueListHead
import uniffi.gemstone.GemHeaderButtonTap
import uniffi.gemstone.GemValueHeader

@Composable
internal fun AssetHeadItem(header: GemValueHeader, onAction: (AssetAction) -> Unit) {
    ValueListHead(header = header) {
        AssetHeadActions(header.actions ?: return@ValueListHead) { tap ->
            when (tap) {
                is GemHeaderButtonTap.Send -> tap.assetId?.toAssetId()?.let { onAction(AssetAction.Transfer(it)) }
                is GemHeaderButtonTap.Receive -> tap.assetId?.toAssetId()?.let { onAction(AssetAction.Receive(it)) }
                is GemHeaderButtonTap.Buy -> tap.assetId?.toAssetId()?.let { onAction(AssetAction.Buy(it)) }
                is GemHeaderButtonTap.Swap -> tap.payAssetId?.toAssetId()?.let { onAction(AssetAction.Swap(fromAssetId = it, toAssetId = tap.receiveAssetId?.toAssetId())) }
                is GemHeaderButtonTap.Deposit, is GemHeaderButtonTap.Withdraw, GemHeaderButtonTap.SendCollectible, GemHeaderButtonTap.CollectibleMenu -> Unit
            }
        }
    }
}
