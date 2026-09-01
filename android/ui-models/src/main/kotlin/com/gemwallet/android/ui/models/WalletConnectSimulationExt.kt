package com.gemwallet.android.ui.models

import com.wallet.core.primitives.BlockExplorerLink
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.SimulationPayloadField
import com.wallet.core.primitives.SimulationPayloadFieldType
import com.wallet.core.primitives.SimulationSeverity
import com.wallet.core.primitives.SimulationWarning
import uniffi.gemstone.GemBlockExplorerLink
import uniffi.gemstone.GemConfirmTransferService
import uniffi.gemstone.GemExplorerService

data class PayloadField(
    val field: SimulationPayloadField,
    val explorerLink: BlockExplorerLink? = null,
    val chain: Chain? = null,
)

fun List<SimulationWarning>.hasCriticalWarning(): Boolean =
    any { it.severity == SimulationSeverity.Critical }

fun List<SimulationPayloadField>.withExplorerLinks(
    chain: Chain?,
    explorerService: GemExplorerService?,
): List<PayloadField> {
    if (chain == null || explorerService == null) return map { PayloadField(field = it, chain = chain) }
    return withAddressLinks(chain) { explorerService.getAddressUrl(chain.string, it) }
}

fun List<SimulationPayloadField>.withExplorerLinks(
    chain: Chain?,
    confirmService: GemConfirmTransferService,
): List<PayloadField> {
    if (chain == null) return map { PayloadField(field = it, chain = null) }
    return withAddressLinks(chain) { confirmService.addressUrl(chain.string, it) }
}

private fun List<SimulationPayloadField>.withAddressLinks(
    chain: Chain,
    addressUrl: (String) -> GemBlockExplorerLink,
): List<PayloadField> = map { field ->
    val link = if (field.fieldType == SimulationPayloadFieldType.Address) {
        addressUrl(field.value).let { BlockExplorerLink(it.name, it.link) }
    } else null
    PayloadField(field = field, explorerLink = link, chain = chain)
}
