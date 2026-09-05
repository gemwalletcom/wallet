package com.gemwallet.android.ui.models

import com.wallet.core.primitives.BlockExplorerLink
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.SimulationPayloadField
import com.wallet.core.primitives.SimulationPayloadFieldType
import uniffi.gemstone.BlockExplorerLink as GemBlockExplorerLink
import com.gemwallet.android.ext.toPrimitives

data class PayloadField(
    val field: SimulationPayloadField,
    val explorerLink: BlockExplorerLink? = null,
    val chain: Chain? = null,
)

fun List<SimulationPayloadField>.withExplorerLinks(
    chain: Chain?,
    addressUrl: (Chain, String) -> GemBlockExplorerLink,
): List<PayloadField> {
    if (chain == null) return map { PayloadField(field = it, chain = null) }
    return map { field ->
        val link = if (field.fieldType == SimulationPayloadFieldType.Address) {
            addressUrl(chain, field.value).toPrimitives()
        } else null
        PayloadField(field = field, explorerLink = link, chain = chain)
    }
}
