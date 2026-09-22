package com.gemwallet.android.features.confirm.viewmodels

import android.content.Context
import com.gemwallet.android.ext.requireChain
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.components.list_head.SimulationHeaderUIModel
import com.gemwallet.android.ui.components.list_head.headerUIModel
import com.gemwallet.android.ui.components.list_item.ListItemImage
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.style.textStyle
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemConfirmSimulationState
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemSimulationBalanceChange
import uniffi.gemstone.GemSimulationPayloadRow
import uniffi.gemstone.GemValueStyle

data class Simulation(
    val warnings: List<GemListRow> = emptyList(),
    val hasCriticalWarning: Boolean = false,
    val primaryPayloadFields: List<GemSimulationPayloadRow> = emptyList(),
    val secondaryPayloadFields: List<GemSimulationPayloadRow> = emptyList(),
    val header: SimulationHeaderUIModel? = null,
    val balanceChanges: List<GemSimulationBalanceChange> = emptyList(),
    val chain: Chain? = null,
)

fun GemConfirmSimulationState.toSimulation(context: Context): Simulation {
    val simulationWarnings = warnings
    val details = simulation ?: return Simulation(warnings = simulationWarnings)

    return Simulation(
        warnings = simulationWarnings,
        hasCriticalWarning = details.hasCriticalWarning,
        primaryPayloadFields = details.primaryFields,
        secondaryPayloadFields = details.secondaryFields,
        header = details.header?.headerUIModel(context),
        balanceChanges = details.balanceChanges,
        chain = chain.requireChain(),
    )
}

fun GemSimulationBalanceChange.formattedValue(): String = sign.amount(value, asset.decimals.toUInt(), asset.symbol, GemValueStyle.FULL).text()

fun GemSimulationBalanceChange.listItem(): ListItemModel = ListItemModel(
    title = asset.name,
    subtitle = formattedValue(),
    subtitleStyle = tone.textStyle(),
    image = ListItemImage.Asset(asset.toPrimitives().id),
)
