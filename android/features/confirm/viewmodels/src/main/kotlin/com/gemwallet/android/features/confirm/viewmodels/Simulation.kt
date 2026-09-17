package com.gemwallet.android.features.confirm.viewmodels

import android.content.Context
import com.gemwallet.android.ext.requireChain
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.ValueFormatter
import com.gemwallet.android.ui.components.list_head.SimulationHeaderUIModel
import com.gemwallet.android.ui.components.list_head.headerUIModel
import com.gemwallet.android.ui.components.list_item.ListItemImage
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.models.PayloadField
import com.gemwallet.android.ui.models.withExplorerLinks
import com.gemwallet.android.ui.style.textStyle
import uniffi.gemstone.GemAmountSign
import uniffi.gemstone.GemConfirmSimulationState
import uniffi.gemstone.GemConfirmationInterface
import uniffi.gemstone.GemSimulationBalanceChange
import uniffi.gemstone.GemSimulationWarningRow
import uniffi.gemstone.GemValueStyle
import uniffi.gemstone.GemValueTone

data class Simulation(
    val warnings: List<GemSimulationWarningRow> = emptyList(),
    val hasCriticalWarning: Boolean = false,
    val primaryPayloadFields: List<PayloadField> = emptyList(),
    val secondaryPayloadFields: List<PayloadField> = emptyList(),
    val header: SimulationHeaderUIModel? = null,
    val balanceChanges: List<GemSimulationBalanceChange> = emptyList(),
)

fun GemConfirmSimulationState.toSimulation(
    session: GemConfirmationInterface,
    context: Context,
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
        header = details.header?.headerUIModel(context),
        balanceChanges = details.balanceChanges,
    )
}

fun GemSimulationBalanceChange.formattedValue(): String =
    sign.format(ValueFormatter(style = GemValueStyle.FULL).string(value.abs(), asset.decimals, asset.symbol))

fun GemSimulationBalanceChange.tone(): GemValueTone = when (sign) {
    GemAmountSign.INCOMING -> GemValueTone.POSITIVE
    GemAmountSign.OUTGOING -> GemValueTone.NEGATIVE
    GemAmountSign.NONE -> GemValueTone.NEUTRAL
}

fun GemSimulationBalanceChange.listItem(): ListItemModel = ListItemModel(
    title = asset.name,
    subtitle = formattedValue(),
    subtitleStyle = tone().textStyle(),
    image = ListItemImage.Asset(asset.toPrimitives().id),
)
