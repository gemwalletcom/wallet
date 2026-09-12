package com.gemwallet.android.features.confirm.viewmodels

import com.gemwallet.android.domains.transaction.format
import uniffi.gemstone.GemAmountSign
import uniffi.gemstone.GemSimulationBalanceChange
import com.gemwallet.android.domains.confirm.ConfirmProperty
import com.gemwallet.android.domains.price.ValueDirection
import uniffi.gemstone.GemSimulationValue
import com.gemwallet.android.model.ValueFormatter
import com.gemwallet.android.ui.models.PayloadField
import com.gemwallet.android.ui.models.withExplorerLinks
import uniffi.gemstone.GemConfirmSimulationState
import uniffi.gemstone.GemConfirmSessionInterface
import com.gemwallet.android.ext.requireChain
import uniffi.gemstone.GemSimulationWarningRow

data class Simulation(
    val warnings: List<GemSimulationWarningRow> = emptyList(),
    val hasCriticalWarning: Boolean = false,
    val primaryPayloadFields: List<PayloadField> = emptyList(),
    val secondaryPayloadFields: List<PayloadField> = emptyList(),
    val header: GemSimulationValue? = null,
    val balanceChanges: List<GemSimulationBalanceChange> = emptyList(),
)

fun GemConfirmSimulationState.toSimulation(
    session: GemConfirmSessionInterface,
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
    sign.format(ValueFormatter(style = ValueFormatter.Style.Full).string(value.abs(), asset.decimals, asset.symbol))

fun GemSimulationBalanceChange.valueDirection(): ValueDirection = when (sign) {
    GemAmountSign.INCOMING -> ValueDirection.Up
    GemAmountSign.OUTGOING -> ValueDirection.Down
    GemAmountSign.NONE -> ValueDirection.None
}

fun List<ConfirmProperty>.reorderRequestProperties(): List<ConfirmProperty> {
    val app = filterIsInstance<ConfirmProperty.Destination.Generic>()
    val wallet = filterIsInstance<ConfirmProperty.Source>()
    val network = filterIsInstance<ConfirmProperty.Network>()

    return buildList {
        addAll(app)
        addAll(wallet)
        addAll(network)
        addAll(
            this@reorderRequestProperties.filterNot {
                it is ConfirmProperty.Destination.Generic
                    || it is ConfirmProperty.Source
                    || it is ConfirmProperty.Network
            }
        )
    }
}
