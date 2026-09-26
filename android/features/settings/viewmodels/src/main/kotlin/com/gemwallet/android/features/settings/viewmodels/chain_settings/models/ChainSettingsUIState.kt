package com.gemwallet.android.features.settings.viewmodels.chain_settings.models

import android.content.Context
import androidx.annotation.StringRes
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.listItemModel
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.localization.text
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemExplorerRow
import uniffi.gemstone.GemNodeRow

data class ChainSettingsUIState(val selectChain: Boolean = true, val chain: Chain? = null, val chains: List<Chain> = emptyList(), val sections: List<ChainSettingsSectionUIModel> = emptyList(), val errorText: String? = null)

sealed class ChainSettingsSectionUIModel(@StringRes val title: Int) {
    data class Nodes(val rows: List<ChainNodeUIModel>) : ChainSettingsSectionUIModel(R.string.settings_networks_source)
    data class Explorers(val rows: List<ExplorerRowUIModel>) : ChainSettingsSectionUIModel(R.string.settings_networks_explorer)
}

data class ChainNodeUIModel(val row: GemNodeRow, val model: ListItemModel)

data class ExplorerRowUIModel(val row: GemExplorerRow, val model: ListItemModel)

internal fun GemNodeRow.uiModel(context: Context): ChainNodeUIModel = ChainNodeUIModel(
    row = this,
    model = latencyStatus.listItemModel(context, title.string(context), subtitle.text(context)),
)

internal fun GemExplorerRow.uiModel() = ExplorerRowUIModel(row = this, model = ListItemModel(title = name))
