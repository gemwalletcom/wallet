package com.gemwallet.android.features.confirm.viewmodels

import com.gemwallet.android.domains.confirm.ConfirmProperty
import com.gemwallet.android.ui.models.PayloadField
import com.gemwallet.android.ui.models.withExplorerLinks
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.SimulationPayloadFieldDisplay
import com.wallet.core.primitives.SimulationPayloadFieldKind
import com.wallet.core.primitives.SimulationResult
import com.wallet.core.primitives.SimulationWarning

data class WalletConnectReview(
    val warnings: List<SimulationWarning> = emptyList(),
    val primaryPayloadFields: List<PayloadField> = emptyList(),
    val secondaryPayloadFields: List<PayloadField> = emptyList(),
    val headerAsset: Asset? = null,
    val headerValue: String? = null,
    val headerIsUnlimited: Boolean = false,
)

fun SimulationResult.toWalletConnectReview(
    chain: Chain? = null,
    explorerName: String? = null,
): WalletConnectReview {
    val hideValueField = header != null
    val filtered = payload.filterNot { hideValueField && it.kind == SimulationPayloadFieldKind.Value }

    return WalletConnectReview(
        warnings = warnings,
        primaryPayloadFields = filtered.filter { it.display == SimulationPayloadFieldDisplay.Primary }
            .withExplorerLinks(chain, explorerName),
        secondaryPayloadFields = filtered.filter { it.display == SimulationPayloadFieldDisplay.Secondary }
            .withExplorerLinks(chain, explorerName),
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
