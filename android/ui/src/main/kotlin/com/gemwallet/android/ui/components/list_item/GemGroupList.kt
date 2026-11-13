package com.gemwallet.android.ui.components.list_item

import androidx.compose.foundation.clickable
import androidx.compose.foundation.lazy.LazyItemScope
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.testTag
import com.gemwallet.android.ui.components.empty.EmptyContentView
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.Spacer16
import uniffi.gemstone.GemGroup
import uniffi.gemstone.GemGroupList
import uniffi.gemstone.GemListItem
import uniffi.gemstone.GemListPhase
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemRowAction

fun LazyListScope.gemGroupList(list: GemGroupList, onSelect: (GemListItem) -> Unit) {
    gemGroups(list.groups) { item, position -> GemListItemView(item = item, listPosition = position, onSelect = onSelect) }
    when (val phase = list.phase) {
        is GemListPhase.Empty -> item(key = "phase:empty") { EmptyContentView(state = phase.state, modifier = Modifier.fillParentMaxSize()) }
        is GemListPhase.Error -> item(key = "phase:error") { GemListRowView(row = GemListRow.Error(phase.error), listPosition = ListPosition.Single) }
        GemListPhase.Rows -> Unit
    }
}

fun LazyListScope.gemGroups(groups: List<GemGroup>, itemContent: @Composable LazyItemScope.(item: GemListItem, position: ListPosition) -> Unit) {
    groups.forEachIndexed { index, group ->
        val key = "group:${group.id ?: "#$index"}"
        val title = group.title
        if (title != null) {
            item(key = key) { SubheaderItem(title.string(LocalContext.current)) }
        } else if (index > 0) {
            item(key = key) { Spacer16() }
        }
        itemsPositioned(group.items, key = { itemIndex, item -> "$key:${item.identity() ?: itemIndex}" }) { position, item ->
            itemContent(item, position)
        }
        gemListSectionFooter(group.footer, key = "$key:footer")
    }
}

val GemGroup.rows: List<GemListRow> get() = items.mapNotNull { (it as? GemListItem.Row)?.row }

fun LazyListScope.gemGroupRows(groups: List<GemGroup>, onSelectAddress: ((String) -> Unit)? = null, onSelect: ((GemRowAction) -> Unit)? = null) {
    gemGroups(groups) { item, position ->
        if (item is GemListItem.Row) {
            GemListRowView(row = item.row, listPosition = position, onSelect = onSelect, onSelectAddress = onSelectAddress)
        }
    }
}

@Composable
private fun GemListItemView(item: GemListItem, listPosition: ListPosition, onSelect: (GemListItem) -> Unit) {
    when (item) {
        is GemListItem.Row -> GemListRowView(row = item.row, listPosition = listPosition)

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

        is GemListItem.Validator -> ValidatorItem(
            data = item.row,
            listPosition = listPosition,
            isSelected = item.isSelected,
            onClick = { onSelect(item) },
        )

        is GemListItem.Connection -> ListItem(
            model = item.connection.row.listItem(),
            listPosition = listPosition,
            modifier = Modifier.clickable { onSelect(item) },
        )

        is GemListItem.PriceAlert -> AssetListItem(
            row = item.item.row.row,
            listPosition = listPosition,
            modifier = Modifier.clickable { onSelect(item) },
        )
    }
}

private fun GemListItem.identity(): String? = when (this) {
    is GemListItem.Row -> null
    is GemListItem.Currency -> row.currency.name
    is GemListItem.Validator -> row.validator.id
    is GemListItem.Connection -> connection.connection.session.id
    is GemListItem.PriceAlert -> item.id
}
