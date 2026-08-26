package com.gemwallet.android.data.repositories.nodes

import com.gemwallet.android.data.service.store.ConfigStore
import com.gemwallet.android.data.service.store.database.NodesDao
import com.gemwallet.android.data.service.store.database.entities.DbNode
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Node
import com.wallet.core.primitives.NodeState
import kotlinx.coroutines.flow.first
import uniffi.gemstone.GemNodeStore

class GemstoneNodeStore(
    private val nodesDao: NodesDao,
    private val configStore: ConfigStore,
) : GemNodeStore {

    override suspend fun getNodes(chain: String): List<String> =
        nodesDao.getNodes(chain.toChain()).first().map { Node(it.url, it.status, it.priority).toJson() }

    override suspend fun addNode(chain: String, node: String) {
        val value = node.decodeJson<Node>()
        nodesDao.addNodes(listOf(DbNode(value.url, value.status, value.priority, chain.toChain())))
    }

    override suspend fun deleteNode(chain: String, url: String) = nodesDao.deleteNode(chain.toChain(), url)

    override suspend fun getSelectedUrl(chain: String): String? =
        configStore.getString(SELECTED_NODE_KEY, chain).takeIf { it.isNotEmpty() }?.decodeJson<Node>()?.url

    override suspend fun setSelectedUrl(chain: String, url: String) =
        configStore.putString(SELECTED_NODE_KEY, Node(url, NodeState.Active, 0).toJson(), chain)

    override suspend fun clearSelectedUrl(chain: String) = configStore.putString(SELECTED_NODE_KEY, "", chain)

    private fun String.toChain(): Chain = Chain.entries.first { it.string == this }

    private companion object {
        const val SELECTED_NODE_KEY = "usage_node"
    }
}
