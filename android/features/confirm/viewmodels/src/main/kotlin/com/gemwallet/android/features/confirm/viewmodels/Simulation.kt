package com.gemwallet.android.features.confirm.viewmodels

import com.gemwallet.android.domains.confirm.ConfirmProperty
import com.gemwallet.android.domains.price.ValueDirection
import com.gemwallet.android.model.ValueFormatter
import com.gemwallet.android.ui.models.PayloadField
import com.gemwallet.android.ui.models.withExplorerLinks
import uniffi.gemstone.GemSimulationFormatter
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.SimulationBalanceChange
import com.wallet.core.primitives.SimulationPayloadField
import com.wallet.core.primitives.SimulationPayloadFieldDisplay
import com.wallet.core.primitives.SimulationPayloadFieldKind
import com.wallet.core.primitives.SimulationResult
import com.wallet.core.primitives.SimulationWarning
import java.math.BigInteger

data class Simulation(
    val warnings: List<SimulationWarning> = emptyList(),
    val primaryPayloadFields: List<PayloadField> = emptyList(),
    val secondaryPayloadFields: List<PayloadField> = emptyList(),
    val headerAsset: Asset? = null,
    val headerValue: String? = null,
    val headerIsUnlimited: Boolean = false,
    val balanceChanges: List<SimulationAssetChange> = emptyList(),
)

data class SimulationAssetChange(
    val asset: Asset,
    val value: BigInteger,
)

fun SimulationResult.toSimulation(
    simulationFormatter: GemSimulationFormatter,
    assets: Map<AssetId, Asset>,
    chain: Chain? = null,
    explorerName: String? = null,
    isApproval: Boolean = false,
): Simulation {
    val showsHeader = isApproval || simulationFormatter.header(toJson()) != null
    val filtered = simulationFormatter.payloadFields(payload.map { it.toJson() }, showsHeader)
        .map { it.decodeJson<SimulationPayloadField>() }

    return Simulation(
        warnings = warnings,
        primaryPayloadFields = filtered.filter { it.display == SimulationPayloadFieldDisplay.Primary }
            .withExplorerLinks(chain, explorerName),
        secondaryPayloadFields = filtered.filter { it.display == SimulationPayloadFieldDisplay.Secondary }
            .withExplorerLinks(chain, explorerName),
        headerValue = header?.value,
        headerIsUnlimited = header?.isUnlimited == true,
        balanceChanges = balanceChanges.toBalanceChanges(assets),
    )
}

fun SimulationAssetChange.formattedValue(): String {
    val formatted = ValueFormatter(style = ValueFormatter.Style.Full).string(value, asset.decimals, asset.symbol)
    return if (value > BigInteger.ZERO) "+$formatted" else formatted
}

fun SimulationAssetChange.valueDirection(): ValueDirection = when {
    value > BigInteger.ZERO -> ValueDirection.Up
    value < BigInteger.ZERO -> ValueDirection.Down
    else -> ValueDirection.None
}

private fun List<SimulationBalanceChange>.toBalanceChanges(assets: Map<AssetId, Asset>): List<SimulationAssetChange> {
    return mapNotNull { change ->
        val value = change.value.toBigIntegerOrNull() ?: return@mapNotNull null
        if (value == BigInteger.ZERO) return@mapNotNull null
        val asset = assets[change.assetId] ?: return@mapNotNull null
        SimulationAssetChange(
            asset = asset,
            value = value,
        )
    }
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
