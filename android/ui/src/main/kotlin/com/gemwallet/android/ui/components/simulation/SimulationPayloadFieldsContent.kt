package com.gemwallet.android.ui.components.simulation

import androidx.compose.foundation.clickable
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.property.AddressPropertyItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.format.rowDateFormatter
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.models.ListPosition
import uniffi.gemstone.GemSimulationPayloadRow
import uniffi.gemstone.GemSimulationPayloadValue
import java.time.ZoneId
import java.util.Locale

fun LazyListScope.simulationPayloadFieldsContent(fields: List<GemSimulationPayloadRow>, onAddressClick: ((String) -> Unit)? = null, onDetailsClick: (() -> Unit)? = null) {
    if (fields.isEmpty() && onDetailsClick == null) {
        return
    }
    val totalItems = fields.size + if (onDetailsClick != null) 1 else 0
    itemsIndexed(fields) { index, row ->
        val listPosition = ListPosition.getPosition(index, totalItems)
        val title = row.title.text(LocalContext.current)
        when (val value = row.value) {
            is GemSimulationPayloadValue.Address -> AddressPropertyItem(
                title = title,
                displayText = value.display,
                copyValue = value.copy.value,
                explorerLink = value.explorer.toPrimitives(),
                listPosition = listPosition,
                onClick = onAddressClick?.let { click -> { click(value.copy.value) } },
            )

            is GemSimulationPayloadValue.Text -> ListItem(
                model = ListItemModel(title = title, subtitle = value.text),
                listPosition = listPosition,
            )

            is GemSimulationPayloadValue.Timestamp -> ListItem(
                model = ListItemModel(title = title, subtitle = LocalContext.current.rowDateFormatter().row(value.unixMs, ZoneId.systemDefault(), Locale.getDefault())),
                listPosition = listPosition,
            )
        }
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

fun LazyListScope.simulationPayloadDetailsContent(primaryFields: List<GemSimulationPayloadRow>, secondaryFields: List<GemSimulationPayloadRow>, onAddressClick: ((String) -> Unit)? = null) {
    simulationPayloadFieldsContent(primaryFields, onAddressClick = onAddressClick)
    if (secondaryFields.isNotEmpty()) {
        item { SubheaderItem(R.string.common_details) }
        simulationPayloadFieldsContent(secondaryFields, onAddressClick = onAddressClick)
    }
}
