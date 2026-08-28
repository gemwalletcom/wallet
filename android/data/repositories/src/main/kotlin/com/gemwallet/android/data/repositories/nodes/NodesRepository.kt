package com.gemwallet.android.data.repositories.nodes

import com.gemwallet.android.cases.nodes.AddNodeCase
import com.gemwallet.android.cases.nodes.DeleteNodeCase
import com.gemwallet.android.cases.nodes.GetCurrentNodeCase
import com.gemwallet.android.cases.nodes.GetNodeUrlCase
import com.gemwallet.android.cases.nodes.GetNodesCase
import com.gemwallet.android.cases.nodes.SetCurrentNodeCase
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Node
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flowOf
import com.gemwallet.android.data.repositories.gemstone.GemstoneNodeStore
import uniffi.gemstone.GemNodeService

class NodesRepository(
    private val nodeService: GemNodeService,
    private val nodeStore: GemstoneNodeStore,
) : SetCurrentNodeCase,
    GetCurrentNodeCase,
    GetNodeUrlCase,
    GetNodesCase,
    AddNodeCase,
    DeleteNodeCase
{

    override suspend fun getNodes(chain: Chain): Flow<List<Node>> =
        flowOf(nodeService.sortedNodes(chain.string, nodeService.getNodes(chain.string)).map { it.decodeJson<Node>() })

    override fun canDeleteNode(chain: Chain, url: String): Boolean = nodeService.canDeleteNode(chain.string, url)

    override fun getDefaultNodes(chain: Chain): List<Node> =
        nodeService.getDefaultNodes(chain.string).map { it.decodeJson<Node>() }

    override suspend fun addNode(chain: Chain, url: String) = nodeService.addNode(chain.string, url)

    override suspend fun deleteNode(chain: Chain, node: Node) = nodeService.deleteNode(chain.string, node.url)

    override suspend fun setCurrentNode(chain: Chain, node: Node) = nodeService.setSelectedNode(chain.string, node.url)

    override fun getCurrentNode(chain: Chain): Node? =
        nodeService.selectedNode(chain.string, nodeStore.selectedUrl(chain), nodeStore.storedNodes(chain)).decodeJson<Node>()

    override fun getNodeUrl(chain: Chain): String =
        nodeService.nodeUrl(chain.string, nodeStore.selectedUrl(chain), nodeStore.storedNodes(chain))

}
