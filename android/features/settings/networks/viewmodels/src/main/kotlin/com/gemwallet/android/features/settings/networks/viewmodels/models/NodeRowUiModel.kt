package com.gemwallet.android.features.settings.networks.viewmodels.models

import uniffi.gemstone.GemNodeStatusState

data class NodeRowUiModel(
    val url: String,
    val host: String,
    val gemNodeFlag: String? = null,
    val selected: Boolean = false,
    val canDelete: Boolean = false,
    val statusState: GemNodeStatusState = GemNodeStatusState.Loading,
) {
    val id: String = url
}
