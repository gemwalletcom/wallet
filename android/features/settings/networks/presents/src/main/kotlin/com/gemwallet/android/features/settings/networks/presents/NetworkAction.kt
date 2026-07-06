package com.gemwallet.android.features.settings.networks.presents

import com.wallet.core.primitives.Node

internal sealed interface NetworkAction {
    data object Refresh : NetworkAction
    data object Cancel : NetworkAction
    data class SelectNode(val node: Node) : NetworkAction
    data class DeleteNode(val node: Node) : NetworkAction
    data class SelectBlockExplorer(val name: String) : NetworkAction
}
