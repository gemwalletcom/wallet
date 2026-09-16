package com.gemwallet.android.features.settings.networks.viewmodels.models

import android.content.Context
import androidx.annotation.StringRes
import com.gemwallet.android.ext.requireChain
import com.gemwallet.android.features.settings.networks.viewmodels.localization.stringRes
import com.gemwallet.android.features.settings.networks.viewmodels.localization.text
import com.gemwallet.android.ui.localization.stringRes
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.buttonState
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemAddNodePhase
import uniffi.gemstone.GemAddNodeSession
import uniffi.gemstone.GemNodeCheckRow
import uniffi.gemstone.GemNodeSyncState

data class AddNodeUIModel(
    val chain: Chain? = null,
    val errorText: String = "",
    val checks: List<NodeCheckRowUIModel> = emptyList(),
    val buttonState: ButtonState = ButtonState.Disabled,
)

data class NodeCheckRowUIModel(
    @StringRes val title: Int,
    val value: String,
    val isInSync: Boolean? = null,
)

internal fun GemAddNodeSession.uiModel(context: Context): AddNodeUIModel {
    val state = viewState()
    val (errorText, checks) = when (val phase = state.phase) {
        is GemAddNodePhase.Idle, is GemAddNodePhase.Checking -> "" to emptyList()
        is GemAddNodePhase.Ready -> "" to phase.check.rows().map { it.uiModel(context) }
        is GemAddNodePhase.Failed -> context.getString(phase.failure.stringRes()) to emptyList()
    }
    return AddNodeUIModel(
        chain = chain.requireChain(),
        errorText = errorText,
        checks = checks,
        buttonState = buttonState(enabled = state.canImport, loading = state.phase is GemAddNodePhase.Checking),
    )
}

private fun GemNodeCheckRow.uiModel(context: Context) = NodeCheckRowUIModel(
    title = stringRes(),
    value = text(context),
    isInSync = when (this) {
        is GemNodeCheckRow.InSync -> when (state) {
            GemNodeSyncState.IN_SYNC -> true
            GemNodeSyncState.OUT_OF_SYNC -> false
        }
        is GemNodeCheckRow.ChainId, is GemNodeCheckRow.LatestBlock, is GemNodeCheckRow.Latency -> null
    },
)
