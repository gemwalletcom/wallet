package com.gemwallet.android.features.confirm.viewmodels

import com.gemwallet.android.domains.confirm.ConfirmProperty
import com.gemwallet.android.domains.price.ValueDirection
import com.gemwallet.android.model.ValueFormatter
import com.gemwallet.android.ui.models.PayloadField
import com.gemwallet.android.ui.models.withExplorerLinks
import uniffi.gemstone.GemApprovalValue
import uniffi.gemstone.GemConfirmSimulationState
import uniffi.gemstone.GemConfirmTransferService
import com.gemwallet.android.ext.toPrimitives
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Chain
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

fun GemConfirmSimulationState.toSimulation(
    warnings: List<SimulationWarning>,
    chain: Chain?,
    confirmService: GemConfirmTransferService,
): Simulation {
    val details = simulation ?: return Simulation(warnings = warnings)
    val header = details.header

    return Simulation(
        warnings = warnings,
        primaryPayloadFields = details.primaryFields.map { it.toPrimitives() }
            .withExplorerLinks(chain) { chain, address -> confirmService.addressUrl(chain.string, address) },
        secondaryPayloadFields = details.secondaryFields.map { it.toPrimitives() }
            .withExplorerLinks(chain) { chain, address -> confirmService.addressUrl(chain.string, address) },
        headerAsset = header?.asset?.toPrimitives(),
        headerValue = (header?.value as? GemApprovalValue.Exact)?.value,
        headerIsUnlimited = header?.value is GemApprovalValue.Unlimited,
        balanceChanges = details.balanceChanges.map { SimulationAssetChange(asset = it.asset.toPrimitives(), value = it.value.toBigInteger()) },
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
