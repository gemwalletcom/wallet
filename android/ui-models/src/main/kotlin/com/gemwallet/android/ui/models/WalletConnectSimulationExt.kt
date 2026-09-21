package com.gemwallet.android.ui.models

import com.gemwallet.android.ext.toPrimitives
import com.wallet.core.primitives.BlockExplorerLink
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemSimulationPayloadRow
import uniffi.gemstone.GemSimulationPayloadValue
import uniffi.gemstone.BlockExplorerLink as GemBlockExplorerLink

data class PayloadField(val row: GemSimulationPayloadRow, val explorerLink: BlockExplorerLink? = null)

fun List<GemSimulationPayloadRow>.withExplorerLinks(chain: Chain?, addressUrl: (Chain, String) -> GemBlockExplorerLink): List<PayloadField> {
    if (chain == null) return map { PayloadField(row = it) }
    return map { row ->
        val link = when (val value = row.value) {
            is GemSimulationPayloadValue.Address -> addressUrl(chain, value.address).toPrimitives()

            is GemSimulationPayloadValue.Text,
            is GemSimulationPayloadValue.Timestamp,
            -> null
        }
        PayloadField(row = row, explorerLink = link)
    }
}
