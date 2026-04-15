package com.gemwallet.android.data.repositories.nodes

import com.gemwallet.android.cases.nodes.AddNodeCase
import com.gemwallet.android.cases.nodes.DeleteNodeCase
import com.gemwallet.android.cases.nodes.GetBlockExplorers
import com.gemwallet.android.cases.nodes.GetCurrentBlockExplorer
import com.gemwallet.android.cases.nodes.GetCurrentNodeCase
import com.gemwallet.android.cases.nodes.GetNodesCase
import com.gemwallet.android.cases.nodes.SetBlockExplorerCase
import com.gemwallet.android.cases.nodes.SetCurrentNodeCase
import com.gemwallet.android.cases.nodes.getGemNode
import com.gemwallet.android.cases.nodes.getGemNodeUrls
import com.gemwallet.android.cases.nodes.getGemNodes
import com.gemwallet.android.cases.nodes.toNode
import com.gemwallet.android.data.service.store.ConfigStore
import com.gemwallet.android.data.service.store.database.NodesDao
import com.gemwallet.android.data.service.store.database.entities.DbNode
import com.gemwallet.android.ext.findByString
import com.gemwallet.android.ext.getSwapMetadata
import com.gemwallet.android.ext.hash
import com.gemwallet.android.model.Transaction
import com.gemwallet.android.serializer.jsonEncoder
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Node
import com.wallet.core.primitives.NodeState
import uniffi.gemstone.GemExplorerInput
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.gemstone.Config
import uniffi.gemstone.Explorer

class NodesRepository(
    private val nodesDao: NodesDao,
    private val configStore: ConfigStore,
    scope: CoroutineScope = CoroutineScope(Dispatchers.IO),
    private val config: Config = Config(),
) : SetCurrentNodeCase,
    GetCurrentNodeCase,
    SetBlockExplorerCase,
    GetBlockExplorers,
    GetCurrentBlockExplorer,
    GetNodesCase,
    AddNodeCase,
    DeleteNodeCase
{

    init {
        scope.launch { sync() }
    }

    override suspend fun getNodes(chain: Chain): Flow<List<Node>> = withContext(Dispatchers.IO) {
        val gemNodes = getGemNodes(chain)
        val configNodeUrls = configNodeUrls(chain)

        nodesDao.getNodes(chain).map { nodes ->
            mergeNodes(
                gemNodes = gemNodes,
                storedNodes = nodes.map { Node(it.url, it.status, it.priority) },
                configNodeUrls = configNodeUrls,
            )
        }
    }

    override suspend fun addNode(chain: Chain, url: String) = withContext(Dispatchers.IO) {
        nodesDao.addNodes(listOf(DbNode(url, NodeState.Active, 0, chain)))
    }

    override suspend fun deleteNode(chain: Chain, node: Node) = withContext(Dispatchers.IO) {
        if (!canDelete(chain, node.url)) {
            return@withContext
        }

        nodesDao.deleteNode(chain, node.url)

        if (getCurrentNode(chain)?.url == node.url) {
            setCurrentNode(chain, getGemNode(chain))
        }
    }

    override fun setCurrentNode(chain: Chain, node: Node) {
        configStore.putString(
            ConfigKey.UsageNode.string,
            jsonEncoder.encodeToString(node),
            chain.string
        )
    }

    override fun getCurrentNode(chain: Chain): Node? {
        val data = configStore.getString(
            ConfigKey.UsageNode.string,
            postfix = chain.string
        )
        val node = try {
            jsonEncoder.decodeFromString<Node>(data)
        } catch (_: Throwable) {
            return null
        }
        return node
    }

    override fun getBlockExplorers(chain: Chain): List<String> {
        return config.getBlockExplorers(chain.string)
    }

    override fun getCurrentBlockExplorer(chain: Chain): String {
        val explorerName = configStore.getString(
            ConfigKey.CurrentExplorer.string,
            chain.string
        )
        val explorers = getBlockExplorers(chain)

        return explorers.firstOrNull { it == explorerName }
            ?: explorers.firstOrNull()
            ?: ""
    }

    override fun getBlockExplorerInfo(transaction: Transaction): Pair<String, String> {
        val chain = transaction.assetId.chain
        val provider = transaction.getSwapMetadata()?.provider

        val blockExplorerName = getCurrentBlockExplorer(chain)
        val explorer = Explorer(chain.string)
        val swapExplorerUrl = provider?.let {
            explorer.getTransactionSwapUrl(
                blockExplorerName,
                GemExplorerInput(
                    hash = transaction.hash,
                    recipient = transaction.to,
                    memo = transaction.memo,
                ),
                provider,
            )
        }
        val explorerUrl = swapExplorerUrl?.url ?: explorer.getTransactionUrl(blockExplorerName, transaction.hash)
        return Pair(
            explorerUrl,
            swapExplorerUrl?.name ?: blockExplorerName,
        )
    }

    override fun setCurrentBlockExplorer(chain: Chain, name: String) {
        configStore.putString(
            ConfigKey.CurrentExplorer.string,
            name,
            chain.string
        )
    }

    private suspend fun sync() {
        val nodes = config.getNodes().mapNotNull { entry ->
            entry.value.mapNotNull {
                val node = it.toNode()
                DbNode(
                    chain = Chain.findByString(entry.key) ?: return@mapNotNull null,
                    url = node.url,
                    status = node.status,
                    priority = node.priority,
                )
            }
        }.flatten()
        nodesDao.addNodes(nodes)
    }

    private fun canDelete(chain: Chain, url: String): Boolean {
        return canDeleteNode(
            url = url,
            gemNodeUrls = getGemNodeUrls(chain),
            configNodeUrls = configNodeUrls(chain),
        )
    }

    private fun configNodeUrls(chain: Chain): Set<String> {
        return config
            .getNodes()[chain.string]
            .orEmpty()
            .mapTo(linkedSetOf()) { it.url }
    }

    private enum class ConfigKey(val string: String) {
        UsageNode("usage_node"),
        CurrentExplorer("current_explorer"),
        ;
    }
}

internal fun mergeNodes(
    gemNodes: List<Node>,
    storedNodes: List<Node>,
    configNodeUrls: Set<String>,
): List<Node> {
    val configNodeOrder = configNodeUrls.withIndex().associate { it.value to it.index }
    val configNodes = storedNodes
        .filter { it.url in configNodeUrls }
        .sortedBy { configNodeOrder[it.url] ?: Int.MAX_VALUE }
    val customNodes = storedNodes.filterNot { it.url in configNodeUrls }

    return (gemNodes + configNodes + customNodes).distinctBy(Node::url)
}

internal fun canDeleteNode(
    url: String,
    gemNodeUrls: Set<String>,
    configNodeUrls: Set<String>,
): Boolean {
    return url !in gemNodeUrls && url !in configNodeUrls
}
