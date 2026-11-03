package com.gemwallet.android.features.price_alerts.presents

import androidx.compose.material3.Switch
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import com.gemwallet.android.ui.components.list_item.AssetListItem
import com.gemwallet.android.ui.models.ListPosition
import uniffi.gemstone.GemAssetItemRow
import uniffi.gemstone.GemPriceAlertItem

@Composable
internal fun PriceAlertAutoAssetItem(row: GemAssetItemRow, enabled: Boolean, onCheckedChange: (Boolean) -> Unit) {
    AssetListItem(
        row = row,
        listPosition = ListPosition.Single,
        accessory = {
            Switch(
                checked = enabled,
                onCheckedChange = onCheckedChange,
                modifier = Modifier.semantics { contentDescription = row.title },
            )
        },
    )
}

@Composable
internal fun PriceAlertItem(item: GemPriceAlertItem, listPosition: ListPosition, modifier: Modifier = Modifier, accessory: (@Composable () -> Unit)?) {
    AssetListItem(
        modifier = modifier,
        row = item.row.row,
        listPosition = listPosition,
        accessory = accessory,
    )
}
