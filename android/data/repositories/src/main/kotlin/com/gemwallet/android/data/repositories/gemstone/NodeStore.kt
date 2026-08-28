package com.gemwallet.android.data.repositories.gemstone

import com.gemwallet.android.data.service.store.ConfigStore
import com.gemwallet.android.data.service.store.database.NodesDao
import com.gemwallet.android.data.service.store.database.entities.DbNode
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Node
import uniffi.gemstone.GemNodeStore
import com.gemwallet.android.ext.requireChain

class GemstoneNodeStore(
    private val nodesDao: NodesDao,
    private val configStore: ConfigStore,
) : GemNodeStore {

    override suspend fun getNodes(chain: String): List<String> = storedNodes(chain.requireChain())

    fun storedNodes(chain: Chain): List<String> =
        nodesDao.getNodeList(chain).map { Node(it.url, it.status, it.priority).toJson() }

    override suspend fun addNode(chain: String, node: String) {
        val value = node.decodeJson<Node>()
        nodesDao.addNodes(listOf(DbNode(value.url, value.status, value.priority, chain.requireChain())))
    }

    override suspend fun deleteNode(chain: String, url: String) = nodesDao.deleteNode(chain.requireChain(), url)

    override suspend fun getSelectedUrl(chain: String): String? = selectedUrl(chain.requireChain())

    fun selectedUrl(chain: Chain): String? = configStore.getString(SELECTED_NODE_KEY, chain.string)
        .takeIf { it.isNotEmpty() }
        ?.let { stored -> if (stored.startsWith("{")) stored.decodeJson<Node>().url else stored }

    override suspend fun setSelectedUrl(chain: String, url: String) = configStore.putString(SELECTED_NODE_KEY, url, chain)

    override suspend fun deleteSelectedUrl(chain: String) = configStore.remove(SELECTED_NODE_KEY, chain)

    private companion object {
        const val SELECTED_NODE_KEY = "usage_node"
    }
}
