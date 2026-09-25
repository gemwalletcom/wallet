package com.gemwallet.android.features.settings.networks.viewmodels.models

import android.content.Context
import androidx.annotation.StringRes
import com.gemwallet.android.ext.requireChain
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.localization.stringRes
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.buttonState
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemAddNodePhase
import uniffi.gemstone.GemAddNodeSession
import uniffi.gemstone.GemNodeCheckRow
import uniffi.gemstone.GemNodeSyncState

data class AddNodeUIModel(val chain: Chain? = null, val errorText: String = "", val checks: List<NodeCheckRowUIModel> = emptyList(), val warning: ListItemModel? = null, val buttonState: ButtonState = ButtonState.Disabled)

data class NodeCheckRowUIModel(val model: ListItemModel, val isInSync: Boolean? = null)

internal fun GemAddNodeSession.uiModel(context: Context): AddNodeUIModel {
    val state = viewState()
    val (errorText, checks) = when (val phase = state.phase) {
        is GemAddNodePhase.Idle, is GemAddNodePhase.Checking -> "" to emptyList()
        is GemAddNodePhase.Ready -> "" to phase.check.rows().map { it.uiModel(context) }
        is GemAddNodePhase.Failed -> phase.error.text(context) to emptyList()
    }
    return AddNodeUIModel(
        chain = chain.requireChain(),
        errorText = errorText,
        checks = checks,
        warning = if (checks.isEmpty()) {
            null
        } else {
            ListItemModel(
                title = context.getString(R.string.asset_verification_warning_title),
                titleExtra = context.getString(R.string.nodes_import_node_warning_message),
            )
        },
        buttonState = buttonState(enabled = state.canImport, loading = state.phase is GemAddNodePhase.Checking),
    )
}

private fun GemNodeCheckRow.uiModel(context: Context) = NodeCheckRowUIModel(
    model = ListItemModel(title = context.getString(stringRes()), subtitle = text(context).takeIf { this !is GemNodeCheckRow.InSync }),
    isInSync = when (this) {
        is GemNodeCheckRow.InSync -> when (state) {
            GemNodeSyncState.IN_SYNC -> true
            GemNodeSyncState.OUT_OF_SYNC -> false
        }

        is GemNodeCheckRow.ChainId, is GemNodeCheckRow.LatestBlock, is GemNodeCheckRow.Latency -> null
    },
)
