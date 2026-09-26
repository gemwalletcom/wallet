package com.gemwallet.android.features.assets.presents.asset.components

import androidx.compose.foundation.clickable
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.testTag
import com.gemwallet.android.features.assets.viewmodels.asset.models.AssetAction
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.components.infoSheet
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.models.ListPosition
import uniffi.gemstone.GemAssetDetailRow
import uniffi.gemstone.GemBalanceRow
import uniffi.gemstone.GemInfoTopic

@Composable
internal fun AssetDetailRowItem(row: GemAssetDetailRow, action: AssetAction?, listPosition: ListPosition, onAction: (AssetAction) -> Unit) {
    val context = LocalContext.current
    when (row) {
        is GemAssetDetailRow.Balance -> ListItem(
            model = ListItemModel(
                title = row.row.row.title().text(context),
                subtitle = row.row.value.text(context),
                info = GemInfoTopic.PendingUnconfirmedBalance.infoSheet().takeIf { row.row.row is GemBalanceRow.PendingUnconfirmed },
            ),
            listPosition = listPosition,
            modifier = action?.let { selected -> Modifier.clickable { onAction(selected) }.testTag("assetStake") } ?: Modifier,
            accessory = action?.let { { DataBadgeChevron() } },
        )

        is GemAssetDetailRow.Row -> GemListRowView(
            row = row.row,
            listPosition = listPosition,
            onSelect = action?.let { selected -> { onAction(selected) } },
            modifier = if (action is AssetAction.OpenChart) Modifier.testTag("assetChart") else Modifier,
        )
    }
}
