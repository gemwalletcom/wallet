package com.gemwallet.android.cases.nodes

import com.wallet.core.primitives.Node
import com.wallet.core.primitives.NodeState
import uniffi.gemstone.NodePriority

fun uniffi.gemstone.Node.toNode(): Node {
    val (status, priority) = priority.toNodeState()
    return Node(
        url = url,
        status = status,
        priority = priority,
    )
}

fun NodePriority.toNodeState(): Pair<NodeState, Int> = when (this) {
    NodePriority.HIGH -> NodeState.Active to 3
    NodePriority.MEDIUM -> NodeState.Active to 2
    NodePriority.LOW -> NodeState.Active to 1
    NodePriority.INACTIVE -> NodeState.Inactive to 0
}
