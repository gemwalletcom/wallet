package com.gemwallet.android.features.wallet_tab.presents.components

import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.runtime.MutableState
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ui.components.list_item.AssetContextActions
import com.gemwallet.android.ui.components.list_item.AssetSectionHeaderItem
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.wallet.core.primitives.AssetId
import uniffi.gemstone.GemAssetSectionKind

private const val AssetsGroupHeaderKeyPrefix = "assets_group_header"

@OptIn(ExperimentalFoundationApi::class)
internal fun LazyListScope.assets(items: List<AssetInfoDataAggregate>, longPressState: MutableState<AssetId?>, group: GemAssetSectionKind, onAssetClick: (AssetId) -> Unit, actions: AssetContextActions) {
    if (items.isEmpty()) return

    item(key = "$AssetsGroupHeaderKeyPrefix-${group.name}") { AssetSectionHeaderItem(group) }

    itemsPositioned(items = items, key = { _, item -> "${item.id.toIdentifier()}-${group.name}" }) { position, item ->
        AssetItem(
            modifier = Modifier.testTag(item.id.toIdentifier()),
            listPosition = position,
            item = item,
            longPressState = longPressState,
            group = group,
            onAssetClick = onAssetClick,
            actions = actions,
        )
    }
}
