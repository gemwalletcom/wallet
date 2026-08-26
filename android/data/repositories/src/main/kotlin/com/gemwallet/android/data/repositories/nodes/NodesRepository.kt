package com.gemwallet.android.data.repositories.nodes

import com.gemwallet.android.cases.nodes.AddNodeCase
import com.gemwallet.android.cases.nodes.DeleteNodeCase
import com.gemwallet.android.cases.nodes.GetBlockExplorers
import com.gemwallet.android.cases.nodes.GetCurrentBlockExplorer
import com.gemwallet.android.cases.nodes.GetCurrentNodeCase
import com.gemwallet.android.cases.nodes.GetNodeUrlCase
import com.gemwallet.android.cases.nodes.GetNodesCase
import com.gemwallet.android.cases.nodes.SetBlockExplorerCase
import com.gemwallet.android.cases.nodes.SetCurrentNodeCase
import com.gemwallet.android.data.service.store.ConfigStore
import com.gemwallet.android.ext.getSwapMetadata
import com.gemwallet.android.ext.hash
import com.wallet.core.primitives.Transaction
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Node
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.runBlocking
import uniffi.gemstone.Config
import uniffi.gemstone.Explorer
import uniffi.gemstone.GemExplorerInput
import uniffi.gemstone.GemNodeService

class NodesRepository(
    private val nodeService: GemNodeService,
    private val configStore: ConfigStore,
    private val config: Config = Config(),
) : SetCurrentNodeCase,
    GetCurrentNodeCase,
    SetBlockExplorerCase,
    GetBlockExplorers,
    GetCurrentBlockExplorer,
    GetNodeUrlCase,
    GetNodesCase,
    AddNodeCase,
    DeleteNodeCase
{

    override suspend fun getNodes(chain: Chain): Flow<List<Node>> =
        flowOf(nodeService.getNodes(chain.string).map { it.decodeJson<Node>() })

    override fun getDefaultNodes(chain: Chain): List<Node> =
        nodeService.getDefaultNodes(chain.string).map { it.decodeJson<Node>() }

    override suspend fun addNode(chain: Chain, url: String) = nodeService.addNode(chain.string, url)

    override suspend fun deleteNode(chain: Chain, node: Node) = nodeService.deleteNode(chain.string, node.url)

    override fun setCurrentNode(chain: Chain, node: Node) = runBlocking {
        nodeService.setSelectedNode(chain.string, node.url)
    }

    override fun getCurrentNode(chain: Chain): Node? = runBlocking {
        nodeService.getSelectedNode(chain.string).decodeJson<Node>()
    }

    override fun getNodeUrl(chain: Chain): String = runBlocking { nodeService.getNodeUrl(chain.string) }

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

    private enum class ConfigKey(val string: String) {
        CurrentExplorer("current_explorer"),
        ;
    }
}
