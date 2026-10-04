package com.gemwallet.android.ui.components.simulation

import androidx.compose.foundation.clickable
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.models.ListPosition
import uniffi.gemstone.GemListRow

fun LazyListScope.simulationPayloadFieldsContent(fields: List<GemListRow>, onAddressClick: ((String) -> Unit)? = null, onDetailsClick: (() -> Unit)? = null) {
    if (fields.isEmpty() && onDetailsClick == null) {
        return
    }
    val totalItems = fields.size + if (onDetailsClick != null) 1 else 0
    itemsPositioned(fields, totalCount = totalItems) { position, row ->
        GemListRowView(row = row, listPosition = position, onSelectAddress = onAddressClick)
    }
    onDetailsClick?.let {
        item {
            ListItem(
                model = ListItemModel(title = stringResource(R.string.common_details)),
                listPosition = ListPosition.getPosition(totalItems - 1, totalItems),
                modifier = Modifier.clickable(onClick = it),
                accessory = { DataBadgeChevron() },
            )
        }
    }
}

fun LazyListScope.simulationPayloadDetailsContent(primaryFields: List<GemListRow>, secondaryFields: List<GemListRow>, onAddressClick: ((String) -> Unit)? = null) {
    simulationPayloadFieldsContent(primaryFields, onAddressClick = onAddressClick)
    if (secondaryFields.isNotEmpty()) {
        item { SubheaderItem(R.string.common_details) }
        simulationPayloadFieldsContent(secondaryFields, onAddressClick = onAddressClick)
    }
}
