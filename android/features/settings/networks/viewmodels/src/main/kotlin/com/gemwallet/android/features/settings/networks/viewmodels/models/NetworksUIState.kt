package com.gemwallet.android.features.settings.networks.viewmodels.models

import android.content.Context
import androidx.annotation.StringRes
import com.gemwallet.android.features.settings.networks.viewmodels.localization.string
import com.gemwallet.android.features.settings.networks.viewmodels.localization.text
import com.gemwallet.android.ui.R
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemExplorerRow
import uniffi.gemstone.GemNodeRow

data class NetworksUIState(
    val selectChain: Boolean = true,
    val chain: Chain? = null,
    val chains: List<Chain> = emptyList(),
    val sections: List<NetworkSectionUIModel> = emptyList(),
    val availableAddNode: Boolean = false,
    val errorText: String? = null,
)

sealed class NetworkSectionUIModel(@StringRes val title: Int) {
    data class Nodes(val rows: List<NodeRowUIModel>) : NetworkSectionUIModel(R.string.settings_networks_source)
    data class Explorers(val rows: List<ExplorerRowUIModel>) : NetworkSectionUIModel(R.string.settings_networks_explorer)
}

data class NodeRowUIModel(
    val url: String,
    val host: String,
    val title: String,
    val subtitle: String,
    val latency: LatencyUIModel,
    val isSelected: Boolean,
    val canDelete: Boolean,
)

data class ExplorerRowUIModel(
    val name: String,
    val isSelected: Boolean,
)

internal fun GemNodeRow.uiModel(context: Context) = NodeRowUIModel(
    url = node.url,
    host = node.host,
    title = title.string(context),
    subtitle = subtitle.text(context),
    latency = latencyStatus.uiModel(context),
    isSelected = node.isSelected,
    canDelete = canDelete,
)

internal fun GemExplorerRow.uiModel() = ExplorerRowUIModel(name = name, isSelected = isSelected)
