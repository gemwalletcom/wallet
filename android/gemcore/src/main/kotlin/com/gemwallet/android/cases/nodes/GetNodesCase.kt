package com.gemwallet.android.cases.nodes

import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Node
import kotlinx.coroutines.flow.Flow

interface GetNodesCase {
    suspend fun getNodes(chain: Chain): Flow<List<Node>>

    fun canDeleteNode(chain: Chain, url: String): Boolean

    fun getDefaultNodes(chain: Chain): List<Node>
}