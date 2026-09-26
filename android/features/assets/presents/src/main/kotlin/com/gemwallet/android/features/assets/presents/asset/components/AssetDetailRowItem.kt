package com.gemwallet.android.features.assets.presents.asset.components

import androidx.compose.foundation.clickable
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import com.gemwallet.android.features.assets.viewmodels.asset.models.AssetAction
import com.gemwallet.android.features.assets.viewmodels.asset.models.AssetDetailRowUIModel
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.models.ListPosition

@Composable
internal fun AssetDetailRowItem(row: AssetDetailRowUIModel, listPosition: ListPosition, onAction: (AssetAction) -> Unit) {
    when (row) {
        is AssetDetailRowUIModel.Balance -> ListItem(
            model = row.model,
            listPosition = listPosition,
            modifier = row.action?.let { action -> Modifier.clickable { onAction(action) }.testTag("assetStake") } ?: Modifier,
            accessory = row.action?.let { { DataBadgeChevron() } },
        )

        is AssetDetailRowUIModel.Row -> GemListRowView(
            row = row.row,
            listPosition = listPosition,
            onSelect = row.action?.let { action -> { onAction(action) } },
            modifier = if (row.action is AssetAction.OpenChart) Modifier.testTag("assetChart") else Modifier,
        )
    }
}
