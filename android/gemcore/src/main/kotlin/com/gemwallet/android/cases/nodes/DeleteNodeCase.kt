package com.gemwallet.android.cases.nodes

import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Node

interface DeleteNodeCase {
    suspend fun deleteNode(chain: Chain, node: Node)
}
