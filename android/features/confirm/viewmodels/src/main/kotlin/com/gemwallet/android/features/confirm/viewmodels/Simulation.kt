package com.gemwallet.android.features.confirm.viewmodels

import uniffi.gemstone.GemAmountSign
import uniffi.gemstone.GemSimulationBalanceChange
import com.gemwallet.android.domains.price.ValueDirection
import uniffi.gemstone.GemSimulationValue
import com.gemwallet.android.model.ValueFormatter
import com.gemwallet.android.ui.models.PayloadField
import com.gemwallet.android.ui.models.withExplorerLinks
import uniffi.gemstone.GemConfirmSimulationState
import uniffi.gemstone.GemConfirmationInterface
import com.gemwallet.android.ext.requireChain
import uniffi.gemstone.GemSimulationWarningRow
import uniffi.gemstone.GemValueStyle

data class Simulation(
    val warnings: List<GemSimulationWarningRow> = emptyList(),
    val hasCriticalWarning: Boolean = false,
    val primaryPayloadFields: List<PayloadField> = emptyList(),
    val secondaryPayloadFields: List<PayloadField> = emptyList(),
    val header: GemSimulationValue? = null,
    val balanceChanges: List<GemSimulationBalanceChange> = emptyList(),
)

fun GemConfirmSimulationState.toSimulation(
    session: GemConfirmationInterface,
): Simulation {
    val simulationWarnings = warnings
    val details = simulation ?: return Simulation(warnings = simulationWarnings)
    val chain = this.chain.requireChain()

    return Simulation(
        warnings = simulationWarnings,
        hasCriticalWarning = details.hasCriticalWarning,
        primaryPayloadFields = details.primaryFields
            .withExplorerLinks(chain) { chain, address -> session.addressUrl(chain.string, address) },
        secondaryPayloadFields = details.secondaryFields
            .withExplorerLinks(chain) { chain, address -> session.addressUrl(chain.string, address) },
        header = details.header,
        balanceChanges = details.balanceChanges,
    )
}

fun GemSimulationBalanceChange.formattedValue(): String =
    sign.format(ValueFormatter(style = GemValueStyle.FULL).string(value.abs(), asset.decimals, asset.symbol))

fun GemSimulationBalanceChange.valueDirection(): ValueDirection = when (sign) {
    GemAmountSign.INCOMING -> ValueDirection.Up
    GemAmountSign.OUTGOING -> ValueDirection.Down
    GemAmountSign.NONE -> ValueDirection.None
}
