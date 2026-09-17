package com.gemwallet.android.features.settings.networks.viewmodels.models

import android.content.Context
import androidx.annotation.StringRes
import com.gemwallet.android.features.settings.networks.viewmodels.localization.string
import com.gemwallet.android.features.settings.networks.viewmodels.localization.text
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.ListItemTextStyle
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
    val isSelected: Boolean,
    val canDelete: Boolean,
    val model: ListItemModel,
)

data class ExplorerRowUIModel(
    val name: String,
    val isSelected: Boolean,
    val model: ListItemModel,
)

internal fun GemNodeRow.uiModel(context: Context): NodeRowUIModel {
    val latency = latencyStatus.uiModel(context)
    return NodeRowUIModel(
        url = node.url,
        host = node.host,
        isSelected = node.isSelected,
        canDelete = canDelete,
        model = ListItemModel(
            title = title.string(context),
            titleTag = latency.text,
            titleTagStyle = latency.tagStyle(),
            titleTagType = latency.tagType(),
            titleExtra = subtitle.text(context),
            titleExtraStyle = ListItemTextStyle.Body,
        ),
    )
}

internal fun GemExplorerRow.uiModel() = ExplorerRowUIModel(name = name, isSelected = isSelected, model = ListItemModel(title = name))
