package com.gemwallet.android.features.price_alerts.presents

import androidx.compose.material3.Switch
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import com.gemwallet.android.features.price_alerts.viewmodels.PriceAlertItemUIModel
import com.gemwallet.android.ui.components.list_item.AssetListItem
import com.gemwallet.android.ui.models.ListPosition
import uniffi.gemstone.GemAssetItemRow

@Composable
internal fun PriceAlertAutoAssetItem(row: GemAssetItemRow, enabled: Boolean, onCheckedChange: (Boolean) -> Unit) {
    AssetListItem(
        row = row,
        listPosition = ListPosition.Single,
        accessory = {
            Switch(
                checked = enabled,
                onCheckedChange = onCheckedChange,
            )
        },
    )
}

@Composable
internal fun PriceAlertAssetItem(item: PriceAlertItemUIModel, listPosition: ListPosition, modifier: Modifier = Modifier) {
    AssetListItem(
        modifier = modifier,
        row = item.row,
        listPosition = listPosition,
    )
}
