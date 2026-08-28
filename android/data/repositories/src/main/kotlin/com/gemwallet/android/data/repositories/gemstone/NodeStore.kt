package com.gemwallet.android.data.repositories.gemstone

import com.gemwallet.android.data.service.store.ConfigStore
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Node
import com.wallet.core.primitives.NodeState
import uniffi.gemstone.GemNodeStore
import com.gemwallet.android.ext.requireChain

class GemstoneNodeStore(
    private val configStore: ConfigStore,
) : GemNodeStore {

    override suspend fun getNodes(chain: String): List<String> = storedNodes(chain.requireChain())

    fun storedNodes(chain: Chain): List<String> = nodes(chain).map { it.toJson() }

    override suspend fun addNode(chain: String, node: String) {
        val value = node.decodeJson<Node>()
        val chainValue = chain.requireChain()
        putNodes(chainValue, nodes(chainValue).filter { it.url != value.url } + value)
    }

    override suspend fun deleteNode(chain: String, url: String) {
        val chainValue = chain.requireChain()
        putNodes(chainValue, nodes(chainValue).filter { it.url != url })
    }

    override suspend fun getSelectedUrl(chain: String): String? = selectedUrl(chain.requireChain())

    fun selectedUrl(chain: Chain): String? =
        configStore.getString(SELECTED_NODE_KEY, chain.string).takeIf { it.isNotEmpty() }?.decodeJson<Node>()?.url

    override suspend fun setSelectedUrl(chain: String, url: String) =
        configStore.putString(SELECTED_NODE_KEY, Node(url, NodeState.Active, 0).toJson(), chain)

    override suspend fun deleteSelectedUrl(chain: String) = configStore.putString(SELECTED_NODE_KEY, "", chain)

    private fun nodes(chain: Chain): List<Node> =
        configStore.getString(NODES_KEY, chain.string).takeIf { it.isNotEmpty() }?.decodeJson<List<Node>>() ?: emptyList()

    private fun putNodes(chain: Chain, nodes: List<Node>) =
        configStore.putString(NODES_KEY, nodes.toJson(), chain.string)

    private companion object {
        const val NODES_KEY = "nodes"
        const val SELECTED_NODE_KEY = "usage_node"
    }
}
