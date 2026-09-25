package com.gemwallet.android.features.perpetuals.presents.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.ui.Modifier
import com.gemwallet.android.features.perpetuals.viewmodels.model.PerpetualPositionRowUIModel
import com.gemwallet.android.ui.components.list_item.AssetListItem
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.models.ListPosition
import uniffi.gemstone.GemAssetItemRow

internal fun LazyListScope.positionProperties(position: GemAssetItemRow?, rows: List<PerpetualPositionRowUIModel>, onAutocloseClick: () -> Unit) {
    if (position == null) {
        return
    }
    item {
        AssetListItem(row = position, listPosition = ListPosition.First)
    }
    itemsIndexed(rows) { index, row ->
        val listPosition = if (index == rows.lastIndex) ListPosition.Last else ListPosition.Middle
        when (row) {
            is PerpetualPositionRowUIModel.Item -> GemListRowView(row = row.row, listPosition = listPosition)

            is PerpetualPositionRowUIModel.Autoclose -> GemListRowView(
                row = row.row,
                listPosition = listPosition,
                modifier = Modifier.clickable(onClick = onAutocloseClick),
                accessory = { DataBadgeChevron() },
            )
        }
    }
}
