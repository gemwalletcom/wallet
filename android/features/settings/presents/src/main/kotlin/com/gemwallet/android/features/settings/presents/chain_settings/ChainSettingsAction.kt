package com.gemwallet.android.features.settings.presents.chain_settings

internal sealed interface ChainSettingsAction {
    data object Refresh : ChainSettingsAction
    data object Cancel : ChainSettingsAction
    data class SelectNode(val url: String) : ChainSettingsAction
    data class DeleteNode(val url: String) : ChainSettingsAction
    data class SelectBlockExplorer(val name: String) : ChainSettingsAction
}
