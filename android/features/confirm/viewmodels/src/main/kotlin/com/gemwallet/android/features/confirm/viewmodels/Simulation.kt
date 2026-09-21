package com.gemwallet.android.features.confirm.viewmodels

import android.content.Context
import com.gemwallet.android.ext.requireChain
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.components.list_head.SimulationHeaderUIModel
import com.gemwallet.android.ui.components.list_head.headerUIModel
import com.gemwallet.android.ui.components.list_item.ListItemImage
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.models.PayloadField
import com.gemwallet.android.ui.models.withExplorerLinks
import com.gemwallet.android.ui.style.textStyle
import uniffi.gemstone.GemConfirmSimulationState
import uniffi.gemstone.GemConfirmationInterface
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemSimulationBalanceChange
import uniffi.gemstone.GemValueStyle

data class Simulation(
    val warnings: List<GemListRow> = emptyList(),
    val hasCriticalWarning: Boolean = false,
    val primaryPayloadFields: List<PayloadField> = emptyList(),
    val secondaryPayloadFields: List<PayloadField> = emptyList(),
    val header: SimulationHeaderUIModel? = null,
    val balanceChanges: List<GemSimulationBalanceChange> = emptyList(),
)

fun GemConfirmSimulationState.toSimulation(session: GemConfirmationInterface, context: Context): Simulation {
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

fun GemSimulationBalanceChange.formattedValue(): String = sign.amount(value, asset.decimals.toUInt(), asset.symbol, GemValueStyle.FULL).text()

fun GemSimulationBalanceChange.listItem(): ListItemModel = ListItemModel(
    title = asset.name,
    subtitle = formattedValue(),
    subtitleStyle = tone.textStyle(),
    image = ListItemImage.Asset(asset.toPrimitives().id),
)
