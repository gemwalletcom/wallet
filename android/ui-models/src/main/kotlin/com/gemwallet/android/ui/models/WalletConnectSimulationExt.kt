package com.gemwallet.android.ui.models

import com.wallet.core.primitives.BlockExplorerLink
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.SimulationPayloadField
import com.wallet.core.primitives.SimulationPayloadFieldType
import com.wallet.core.primitives.SimulationSeverity
import com.wallet.core.primitives.SimulationWarning
import uniffi.gemstone.Explorer

data class PayloadField(
    val field: SimulationPayloadField,
    val explorerLink: BlockExplorerLink? = null,
)

fun List<SimulationWarning>.hasCriticalWarning(): Boolean =
    any { it.severity == SimulationSeverity.Critical }

fun List<SimulationPayloadField>.withExplorerLinks(
    chain: Chain?,
    explorerName: String?,
): List<PayloadField> {
    if (chain == null || explorerName == null) return map { PayloadField(it) }
    val explorer = Explorer(chain.string)
    return map { field ->
        val link = if (field.fieldType == SimulationPayloadFieldType.Address) {
            BlockExplorerLink(explorerName, explorer.getAddressUrl(explorerName, field.value))
        } else null
        PayloadField(field, link)
    }
}
