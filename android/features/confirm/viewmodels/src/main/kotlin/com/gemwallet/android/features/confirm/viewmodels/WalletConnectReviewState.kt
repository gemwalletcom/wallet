package com.gemwallet.android.features.confirm.viewmodels

import com.gemwallet.android.domains.confirm.ConfirmProperty
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.SimulationHeader
import com.wallet.core.primitives.SimulationPayloadField
import com.wallet.core.primitives.SimulationPayloadFieldDisplay
import com.wallet.core.primitives.SimulationPayloadFieldKind
import com.wallet.core.primitives.SimulationResult
import com.wallet.core.primitives.SimulationWarning

data class WalletConnectReview(
    val warnings: List<SimulationWarning> = emptyList(),
    val primaryPayloadFields: List<SimulationPayloadField> = emptyList(),
    val secondaryPayloadFields: List<SimulationPayloadField> = emptyList(),
    val headerAsset: Asset? = null,
    val headerValue: String? = null,
    val headerIsUnlimited: Boolean = false,
)

fun SimulationResult.toWalletConnectReview(): WalletConnectReview {
    val hideValueField = header != null
    val filtered = payload.filterNot { hideValueField && it.kind == SimulationPayloadFieldKind.Value }

    return WalletConnectReview(
        warnings = warnings,
        primaryPayloadFields = filtered.filter { it.display == SimulationPayloadFieldDisplay.Primary },
        secondaryPayloadFields = filtered.filter { it.display == SimulationPayloadFieldDisplay.Secondary },
        headerValue = header?.value,
        headerIsUnlimited = header?.isUnlimited == true,
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
