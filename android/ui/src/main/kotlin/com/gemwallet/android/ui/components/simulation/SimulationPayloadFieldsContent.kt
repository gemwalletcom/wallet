package com.gemwallet.android.ui.components.simulation

import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.itemsIndexed
import com.gemwallet.android.ext.AddressFormatter
import com.gemwallet.android.ext.secondsToMillis
import com.gemwallet.android.math.getRelativeDate
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.property.AddressPropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.PayloadField
import com.wallet.core.primitives.SimulationPayloadField
import com.wallet.core.primitives.SimulationPayloadFieldKind
import com.wallet.core.primitives.SimulationPayloadFieldType
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
        val titleRes = fieldTitleRes(field)
        when {
            titleRes != null && field.fieldType == SimulationPayloadFieldType.Address -> AddressPropertyItem(
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

private fun fieldTitleRes(field: SimulationPayloadField): Int? = when (field.kind) {
    SimulationPayloadFieldKind.Contract -> R.string.asset_contract
    SimulationPayloadFieldKind.Method -> R.string.common_method
    SimulationPayloadFieldKind.Token -> R.string.common_token
    SimulationPayloadFieldKind.Spender -> R.string.transfer_to
    SimulationPayloadFieldKind.Value -> R.string.perpetual_value
    else -> null
}

private fun fieldValue(payload: PayloadField, addressNames: Map<String, String>): String = when (payload.field.fieldType) {
    SimulationPayloadFieldType.Address -> addressDisplay(payload, addressNames)
    SimulationPayloadFieldType.Timestamp -> payload.field.value.toTimestampText()
    else -> payload.field.value
}

private fun addressDisplay(payload: PayloadField, addressNames: Map<String, String>): String {
    val address = AddressFormatter(payload.field.value, chain = payload.chain).value()
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
