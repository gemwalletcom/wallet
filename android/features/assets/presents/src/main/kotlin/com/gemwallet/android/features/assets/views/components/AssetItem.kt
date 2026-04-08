package com.gemwallet.android.features.assets.views.components

import androidx.compose.runtime.Composable
import androidx.compose.runtime.MutableState
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ui.components.list_item.AssetContextActions
import com.gemwallet.android.ui.components.list_item.AssetContextMenuRow
import com.gemwallet.android.ui.components.list_item.AssetListItem
import com.gemwallet.android.ui.models.AssetsGroupType
import com.gemwallet.android.ui.models.ListPosition
import com.wallet.core.primitives.AssetId

@Composable
internal fun AssetItem(
    listPosition: ListPosition,
    item: AssetInfoDataAggregate,
    longPressState: MutableState<AssetId?>,
    modifier: Modifier = Modifier,
    group: AssetsGroupType = AssetsGroupType.None,
    onAssetClick: (AssetId) -> Unit,
    actions: AssetContextActions,
) {
    AssetContextMenuRow(
        assetId = item.id,
        address = item.accountAddress,
        isPinned = group == AssetsGroupType.Pined,
        isEnabled = true,
        longPressed = longPressState,
        actions = actions,
        onClick = { onAssetClick(item.id) },
        modifier = modifier.testTag(item.id.toIdentifier()),
    ) { rowModifier ->
        AssetListItem(asset = item, listPosition = listPosition, modifier = rowModifier)
    }
}
