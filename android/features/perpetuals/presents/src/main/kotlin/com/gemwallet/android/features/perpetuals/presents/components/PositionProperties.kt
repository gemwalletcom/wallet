package com.gemwallet.android.features.perpetuals.presents.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.ui.Modifier
import com.gemwallet.android.ui.components.list_item.AssetListItem
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.models.ListPosition
import uniffi.gemstone.GemAssetItemRow
import uniffi.gemstone.GemPerpetualPositionDetail
import uniffi.gemstone.GemPerpetualPositionDetailRow

internal fun LazyListScope.positionProperties(position: GemAssetItemRow?, rows: List<GemPerpetualPositionDetail>, onAutocloseClick: () -> Unit) {
    if (position == null) {
        return
    }
    item {
        AssetListItem(row = position, listPosition = ListPosition.First)
    }
    itemsIndexed(rows) { index, row ->
        val listPosition = if (index == rows.lastIndex) ListPosition.Last else ListPosition.Middle
        when (row.kind) {
            GemPerpetualPositionDetailRow.AUTOCLOSE -> GemListRowView(
                row = row.row,
                listPosition = listPosition,
                modifier = Modifier.clickable(onClick = onAutocloseClick),
                accessory = { DataBadgeChevron() },
            )

            GemPerpetualPositionDetailRow.PNL,
            GemPerpetualPositionDetailRow.SIZE,
            GemPerpetualPositionDetailRow.ENTRY_PRICE,
            GemPerpetualPositionDetailRow.LIQUIDATION_PRICE,
            GemPerpetualPositionDetailRow.MARGIN,
            GemPerpetualPositionDetailRow.FUNDING_PAYMENTS,
            -> GemListRowView(row = row.row, listPosition = listPosition)
        }
    }
}
