package com.gemwallet.android.ui.components.simulation

import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.itemsIndexed
import com.gemwallet.android.ext.AddressFormatter
import com.gemwallet.android.math.getRelativeDate
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.models.ListPosition
import com.wallet.core.primitives.SimulationPayloadField
import com.wallet.core.primitives.SimulationPayloadFieldKind
import com.wallet.core.primitives.SimulationPayloadFieldType
import java.time.Instant

fun LazyListScope.simulationPayloadFieldsContent(
    fields: List<SimulationPayloadField>,
    onDetailsClick: (() -> Unit)? = null,
) {
    if (fields.isEmpty() && onDetailsClick == null) {
        return
    }
    val totalItems = fields.size + if (onDetailsClick != null) 1 else 0
    itemsIndexed(fields) { index, field ->
        val listPosition = ListPosition.getPosition(index, totalItems)
        fieldTitleRes(field)?.let { titleRes ->
            PropertyItem(
                title = titleRes,
                data = fieldValue(field),
                listPosition = listPosition,
            )
        } ?: PropertyItem(
            title = field.label.orEmpty(),
            data = fieldValue(field),
            listPosition = listPosition,
        )
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
    primaryFields: List<SimulationPayloadField>,
    secondaryFields: List<SimulationPayloadField>,
) {
    simulationPayloadFieldsContent(primaryFields)
    if (secondaryFields.isNotEmpty()) {
        item { SubheaderItem(R.string.common_details) }
        simulationPayloadFieldsContent(secondaryFields)
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

private fun fieldValue(field: SimulationPayloadField): String = when (field.fieldType) {
    SimulationPayloadFieldType.Address -> AddressFormatter(field.value).value()
    SimulationPayloadFieldType.Timestamp -> field.value.toTimestampText()
    else -> field.value
}

private fun String.toTimestampText(): String {
    toLongOrNull()?.let { return getRelativeDate(it * 1000) }
    return runCatching {
        getRelativeDate(Instant.parse(this).toEpochMilli())
    }.getOrElse {
        this
    }
}
