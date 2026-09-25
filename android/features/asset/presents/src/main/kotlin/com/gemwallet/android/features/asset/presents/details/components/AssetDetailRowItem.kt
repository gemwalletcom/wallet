package com.gemwallet.android.features.asset.presents.details.components

import androidx.compose.foundation.clickable
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import com.gemwallet.android.features.asset.viewmodels.details.models.AssetDetailsAction
import com.gemwallet.android.features.asset.viewmodels.details.models.AssetInfoUIModel
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.models.ListPosition

@Composable
internal fun AssetDetailRowItem(row: AssetInfoUIModel.RowUIModel, listPosition: ListPosition, onAction: (AssetDetailsAction) -> Unit) {
    when (row) {
        is AssetInfoUIModel.RowUIModel.Balance -> ListItem(
            model = row.model,
            listPosition = listPosition,
            modifier = row.action?.let { action -> Modifier.clickable { onAction(action) }.testTag("assetStake") } ?: Modifier,
            accessory = row.action?.let { { DataBadgeChevron() } },
        )

        is AssetInfoUIModel.RowUIModel.Row -> GemListRowView(
            row = row.row,
            listPosition = listPosition,
            onSelect = row.action?.let { action -> { onAction(action) } },
            modifier = if (row.action is AssetDetailsAction.OpenChart) Modifier.testTag("assetChart") else Modifier,
        )
    }
}
