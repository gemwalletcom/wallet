package com.gemwallet.android.features.settings.networks.presents

internal sealed interface NetworkAction {
    data object Refresh : NetworkAction
    data object Cancel : NetworkAction
    data class SelectNode(val url: String) : NetworkAction
    data class DeleteNode(val url: String) : NetworkAction
    data class SelectBlockExplorer(val name: String) : NetworkAction
}
