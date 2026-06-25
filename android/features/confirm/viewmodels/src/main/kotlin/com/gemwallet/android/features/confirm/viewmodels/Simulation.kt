package com.gemwallet.android.features.confirm.viewmodels

import com.gemwallet.android.domains.confirm.ConfirmProperty
import com.gemwallet.android.domains.price.ValueDirection
import com.gemwallet.android.ext.assetType
import com.gemwallet.android.ext.truncate
import com.gemwallet.android.model.ValueFormatter
import com.gemwallet.android.ui.models.PayloadField
import com.gemwallet.android.ui.models.withExplorerLinks
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.SimulationBalanceChange
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
    chain: Chain? = null,
    explorerName: String? = null,
    balanceChangeAssets: List<Asset> = emptyList(),
): Simulation {
    val hideValueField = header != null
    val filtered = payload.filterNot { hideValueField && it.kind == SimulationPayloadFieldKind.Value }

    return Simulation(
        warnings = warnings,
        primaryPayloadFields = filtered.filter { it.display == SimulationPayloadFieldDisplay.Primary }
            .withExplorerLinks(chain, explorerName),
        secondaryPayloadFields = filtered.filter { it.display == SimulationPayloadFieldDisplay.Secondary }
            .withExplorerLinks(chain, explorerName),
        headerValue = header?.value,
        headerIsUnlimited = header?.isUnlimited == true,
        balanceChanges = balanceChanges.toBalanceChanges(balanceChangeAssets),
    )
}

fun SimulationAssetChange.formattedValue(): String {
    val formatted = ValueFormatter(style = ValueFormatter.Style.Full).string(value, asset)
    return if (value > BigInteger.ZERO) "+$formatted" else formatted
}

fun SimulationAssetChange.valueDirection(): ValueDirection = when {
    value > BigInteger.ZERO -> ValueDirection.Up
    value < BigInteger.ZERO -> ValueDirection.Down
    else -> ValueDirection.None
}

private fun List<SimulationBalanceChange>.toBalanceChanges(assets: List<Asset>): List<SimulationAssetChange> {
    val assetsById = assets.associateBy { it.id }
    return mapNotNull { change ->
        val value = change.value.toBigIntegerOrNull() ?: return@mapNotNull null
        if (value == BigInteger.ZERO) return@mapNotNull null
        val asset = assetsById[change.assetId] ?: change.assetId.unresolvedSimulationAsset()
        SimulationAssetChange(asset = asset, value = value)
    }
}

private fun AssetId.unresolvedSimulationAsset(): Asset {
    val identifier = tokenId?.truncate() ?: chain.string
    val type = if (tokenId == null) AssetType.NATIVE else (chain.assetType() ?: AssetType.TOKEN)
    return Asset(
        id = this,
        name = identifier,
        symbol = identifier,
        decimals = 0,
        type = type,
    )
}

fun List<ConfirmProperty>.reorderWalletConnectProperties(): List<ConfirmProperty> {
    val app = filterIsInstance<ConfirmProperty.Destination.Generic>()
    val wallet = filterIsInstance<ConfirmProperty.Source>()
    val network = filterIsInstance<ConfirmProperty.Network>()

    return buildList {
        addAll(app)
        addAll(wallet)
        addAll(network)
        addAll(
            this@reorderWalletConnectProperties.filterNot {
                it is ConfirmProperty.Destination.Generic
                    || it is ConfirmProperty.Source
                    || it is ConfirmProperty.Network
            }
        )
    }
}
