package com.gemwallet.android.ui.components.simulation

import com.gemwallet.android.ui.localization.stringRes
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.runtime.Composable
import com.gemwallet.android.ui.format.rememberFormattedAddress
import com.gemwallet.android.ext.secondsToMillis
import com.gemwallet.android.math.getRelativeDate
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.property.AddressPropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.PayloadField
import uniffi.gemstone.SimulationPayloadField
import uniffi.gemstone.SimulationPayloadFieldKind
import uniffi.gemstone.SimulationPayloadFieldType
import java.time.Instant

fun LazyListScope.simulationPayloadFieldsContent(
    fields: List<PayloadField>,
    addressNames: Map<String, String> = emptyMap(),
    onDetailsClick: (() -> Unit)? = null,
) {
    if (fields.isEmpty() && onDetailsClick == null) {
        return
    }
    val totalItems = fields.size + if (onDetailsClick != null) 1 else 0
    itemsIndexed(fields) { index, payload ->
        val listPosition = ListPosition.getPosition(index, totalItems)
        val field = payload.field
        val titleRes = field.kind.stringRes()
        when {
            titleRes != null && field.fieldType == SimulationPayloadFieldType.ADDRESS -> AddressPropertyItem(
                title = titleRes,
                displayText = addressDisplay(payload, addressNames),
                copyValue = field.value,
                explorerLink = payload.explorerLink,
                listPosition = listPosition,
            )
            titleRes != null -> PropertyItem(
                title = titleRes,
                data = fieldValue(payload, addressNames),
                listPosition = listPosition,
            )
            else -> PropertyItem(
                title = field.label.orEmpty(),
                data = fieldValue(payload, addressNames),
                listPosition = listPosition,
            )
        }
    }
    onDetailsClick?.let {
        item {
            PropertyItem(
                action = R.string.common_details,
                listPosition = ListPosition.getPosition(totalItems - 1, totalItems),
                onClick = it,
            )
        }
    }
}

fun LazyListScope.simulationPayloadDetailsContent(
    primaryFields: List<PayloadField>,
    secondaryFields: List<PayloadField>,
    addressNames: Map<String, String> = emptyMap(),
) {
    simulationPayloadFieldsContent(primaryFields, addressNames)
    if (secondaryFields.isNotEmpty()) {
        item { SubheaderItem(R.string.common_details) }
        simulationPayloadFieldsContent(secondaryFields, addressNames)
    }
}

@Composable
private fun fieldValue(payload: PayloadField, addressNames: Map<String, String>): String = when (payload.field.fieldType) {
    SimulationPayloadFieldType.ADDRESS -> addressDisplay(payload, addressNames)
    SimulationPayloadFieldType.TIMESTAMP -> payload.field.value.toTimestampText()
    SimulationPayloadFieldType.TEXT -> payload.field.value
}

@Composable
private fun addressDisplay(payload: PayloadField, addressNames: Map<String, String>): String {
    val address = rememberFormattedAddress(payload.field.value, payload.chain)
    val name = addressNames[payload.field.value.lowercase()]
    return if (name.isNullOrEmpty()) address else "$name ($address)"
}

private fun String.toTimestampText(): String {
    toLongOrNull()?.let { return getRelativeDate(it.secondsToMillis()) }
    return runCatching {
        getRelativeDate(Instant.parse(this).toEpochMilli())
    }.getOrElse {
        this
    }
}
