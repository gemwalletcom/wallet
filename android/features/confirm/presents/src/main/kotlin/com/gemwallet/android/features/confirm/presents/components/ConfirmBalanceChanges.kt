package com.gemwallet.android.features.confirm.presents.components

import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.text.style.TextOverflow
import com.gemwallet.android.features.confirm.viewmodels.SimulationAssetChange
import com.gemwallet.android.features.confirm.viewmodels.formattedValue
import com.gemwallet.android.features.confirm.viewmodels.valueDirection
import com.gemwallet.android.ui.components.image.AssetIcon
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.color
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.smallIconSize

fun LazyListScope.confirmBalanceChangesContent(changes: List<SimulationAssetChange>) {
    itemsIndexed(changes) { index, change ->
        ConfirmBalanceChangeItem(
            change = change,
            listPosition = ListPosition.getPosition(index, changes.size),
        )
    }
}

@Composable
private fun ConfirmBalanceChangeItem(change: SimulationAssetChange, listPosition: ListPosition) {
    ListItem(
        listPosition = listPosition,
        leading = {
            AssetIcon(asset = change.asset, size = smallIconSize)
        },
        title = {
            Text(
                text = change.asset.name,
                style = MaterialTheme.typography.bodyLarge,
                color = MaterialTheme.colorScheme.onSurface,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        },
        trailing = {
            Text(
                text = change.formattedValue(),
                style = MaterialTheme.typography.bodyLarge,
                color = change.valueDirection().color(),
                maxLines = 1,
            )
        },
    )
}
