package com.gemwallet.android.features.settings.networks.viewmodels.models

import uniffi.gemstone.GemNodeSelection
import uniffi.gemstone.GemNodeStatusState

data class NodeRowUiModel(
    val node: GemNodeSelection,
    val canDelete: Boolean = false,
    val statusState: GemNodeStatusState = GemNodeStatusState.Loading,
) {
    val id: String = node.url

    val url: String = node.url

    val selected: Boolean = node.isSelected
}
