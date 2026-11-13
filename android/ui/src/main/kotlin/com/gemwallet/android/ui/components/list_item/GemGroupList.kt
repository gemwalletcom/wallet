package com.gemwallet.android.ui.components.list_item

import androidx.compose.foundation.clickable
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import com.gemwallet.android.ui.components.empty.EmptyContentView
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.localization.titleRes
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.Spacer16
import uniffi.gemstone.GemGroupList
import uniffi.gemstone.GemListItem
import uniffi.gemstone.GemListPhase
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemListSectionTitle

fun LazyListScope.gemGroupList(list: GemGroupList, onSelect: (GemListItem) -> Unit) {
    list.groups.forEachIndexed { index, group ->
        val key = if (group.title == GemListSectionTitle.NONE) "group:$index" else "group:${group.title.name}"
        val title = group.title.titleRes()
        if (title != null) {
            item(key = key) { SubheaderItem(title) }
        } else if (index > 0) {
            item(key = key) { Spacer16() }
        }
        itemsPositioned(group.items, key = { itemIndex, item -> item.identity() ?: "$key:$itemIndex" }) { position, item ->
            GemListItemView(item = item, listPosition = position, onSelect = onSelect)
        }
        gemListSectionFooter(group.footer, key = "$key:footer")
    }
    when (val phase = list.phase) {
        is GemListPhase.Empty -> item(key = "phase:empty") { EmptyContentView(state = phase.state, modifier = Modifier.fillParentMaxSize()) }
        is GemListPhase.Error -> item(key = "phase:error") { GemListRowView(row = GemListRow.Error(phase.error), listPosition = ListPosition.Single) }
        GemListPhase.Rows -> Unit
    }
}

@Composable
private fun GemListItemView(item: GemListItem, listPosition: ListPosition, onSelect: (GemListItem) -> Unit) {
    when (item) {
        is GemListItem.Currency -> ListItem(
            model = ListItemModel(title = item.row.title),
            listPosition = listPosition,
            modifier = Modifier
                .clickable { onSelect(item) }
                .testTag(item.row.currency.name),
            minHeight = ListItemDefaults.plainMinHeight,
            accessory = if (item.row.isSelected) {
                { SelectionCheckmark() }
            } else {
                null
            },
        )
    }
}

private fun GemListItem.identity(): String? = when (this) {
    is GemListItem.Currency -> row.currency.name
}
