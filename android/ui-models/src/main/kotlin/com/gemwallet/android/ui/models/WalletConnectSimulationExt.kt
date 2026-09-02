package com.gemwallet.android.ui.models

import com.wallet.core.primitives.BlockExplorerLink
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.SimulationPayloadField
import com.wallet.core.primitives.SimulationPayloadFieldType
import com.wallet.core.primitives.SimulationSeverity
import com.wallet.core.primitives.SimulationWarning
import uniffi.gemstone.GemBlockExplorerLink

data class PayloadField(
    val field: SimulationPayloadField,
    val explorerLink: BlockExplorerLink? = null,
    val chain: Chain? = null,
)

fun List<SimulationWarning>.hasCriticalWarning(): Boolean =
    any { it.severity == SimulationSeverity.Critical }

fun List<SimulationPayloadField>.withExplorerLinks(
    chain: Chain?,
    addressUrl: (Chain, String) -> GemBlockExplorerLink,
): List<PayloadField> {
    if (chain == null) return map { PayloadField(field = it, chain = null) }
    return map { field ->
        val link = if (field.fieldType == SimulationPayloadFieldType.Address) {
            addressUrl(chain, field.value).let { BlockExplorerLink(it.name, it.link) }
        } else null
        PayloadField(field = field, explorerLink = link, chain = chain)
    }
}
