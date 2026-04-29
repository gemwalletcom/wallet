package com.gemwallet.android.features.settings.networks.viewmodels

import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.runtime.snapshotFlow
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.blockchain.services.NodeStatusService
import com.gemwallet.android.cases.nodes.DeleteNodeCase
import com.gemwallet.android.cases.nodes.GetBlockExplorers
import com.gemwallet.android.cases.nodes.GetCurrentBlockExplorer
import com.gemwallet.android.cases.nodes.GetCurrentNodeCase
import com.gemwallet.android.cases.nodes.GetNodesCase
import com.gemwallet.android.cases.nodes.SetBlockExplorerCase
import com.gemwallet.android.cases.nodes.SetCurrentNodeCase
import com.gemwallet.android.cases.nodes.getGemNode
import com.gemwallet.android.cases.nodes.getGemNodeRegion
import com.gemwallet.android.cases.nodes.getGemNodeUrls
import com.gemwallet.android.data.repositories.chains.ChainInfoRepository
import com.gemwallet.android.ext.filter
import com.gemwallet.android.model.NodeStatus
import com.gemwallet.android.features.settings.networks.viewmodels.models.NodeRowUiModel
import com.gemwallet.android.features.settings.networks.viewmodels.models.NodeStatusState
import com.gemwallet.android.features.settings.networks.viewmodels.models.NetworksUIState
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Node
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.supervisorScope
import kotlinx.coroutines.withContext
import java.net.URI
import javax.inject.Inject
import uniffi.gemstone.Config

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class NetworksViewModel @Inject constructor(
    private val chainInfoRepository: ChainInfoRepository,
    private val getNodesCase: GetNodesCase,
    private val getCurrentBlockExplorer: GetCurrentBlockExplorer,
    private val getBlockExplorers: GetBlockExplorers,
    private val setBlockExplorerCase: SetBlockExplorerCase,
    private val getCurrentNodeCase: GetCurrentNodeCase,
    private val setCurrentNodeCase: SetCurrentNodeCase,
    private val deleteNodeCase: DeleteNodeCase,
    private val nodeStatusClient: NodeStatusService,
    private val config: Config,
) : ViewModel() {

    private val state = MutableStateFlow(State())
    private val _uiState = MutableStateFlow(state.value.toUIState())
    val uiState = _uiState.asStateFlow()
    val chainFilter = TextFieldState()

    private var observeNodesJob: Job? = null
    private var refreshJob: Job? = null

    init {
        viewModelScope.launch {
            updateState { it.copy(availableChains = chainInfoRepository.getAll()) }
            snapshotFlow { chainFilter.text }.collectLatest { query ->
                updateState { it.copy(availableChains = chainInfoRepository.getAll().filter(query.toString().lowercase())) }
            }
        }
    }

    fun onSelectedChain(chain: Chain) {
        val defaultNodeUrls = config
            .getNodes()[chain.string]
            .orEmpty()
            .mapTo(linkedSetOf()) { it.url }

        updateState {
            it.copy(
                chain = chain,
                selectChain = false,
                explorers = getBlockExplorers.getBlockExplorers(chain),
                currentNode = getCurrentNodeCase.getCurrentNode(chain),
                currentExplorer = getCurrentBlockExplorer.getCurrentBlockExplorer(chain),
                availableAddNode = true,
                defaultNodeUrls = defaultNodeUrls,
                nodes = emptyList(),
                nodeRows = emptyList(),
                nodeStates = emptyMap(),
                isRefreshing = true,
                refreshNonce = System.nanoTime(),
            )
        }
        observeNodes(chain)
    }

    fun refresh() {
        val chain = state.value.chain ?: return
        refreshNodeStatuses(chain, System.nanoTime())
    }

    fun onSelectNode(node: Node) {
        val chain = state.value.chain ?: return
        setCurrentNodeCase.setCurrentNode(chain, node)
        updateState {
            it.copy(
                currentNode = node,
                nodeRows = buildNodeRows(
                    chain = chain,
                    nodes = it.nodes,
                    currentNode = node,
                    nodeStates = it.nodeStates,
                    defaultNodeUrls = it.defaultNodeUrls,
                ),
            )
        }
    }

    fun onSelectBlockExplorer(name: String) {
        val chain = state.value.chain ?: return
        setBlockExplorerCase.setCurrentBlockExplorer(chain, name)
        updateState { it.copy(currentExplorer = name) }
    }

    fun onSelectChain() {
        updateState { it.copy(selectChain = true) }
    }

    fun onDeleteNode(node: Node) {
        val chain = state.value.chain ?: return
        viewModelScope.launch {
            deleteNodeCase.deleteNode(chain, node)
            updateState {
                val nodes = it.nodes.filterNot { currentNode -> currentNode.url == node.url }
                val nodeStates = visibleNodeStates(nodes, it.nodeStates)
                val currentNode = currentNodeFor(chain, nodes, it.currentNode)
                it.copy(
                    nodes = nodes,
                    currentNode = currentNode,
                    nodeStates = nodeStates,
                    nodeRows = buildNodeRows(
                        chain = chain,
                        nodes = nodes,
                        currentNode = currentNode,
                        nodeStates = nodeStates,
                        defaultNodeUrls = it.defaultNodeUrls,
                    ),
                )
            }
        }
    }

    private fun observeNodes(chain: Chain) {
        observeNodesJob?.cancel()
        observeNodesJob = viewModelScope.launch {
            getNodesCase.getNodes(chain).collectLatest { nodes ->
                val currentNode = currentNodeFor(chain, nodes, state.value.currentNode)
                val currentStates = visibleNodeStates(nodes, state.value.nodeStates)

                updateState {
                    it.copy(
                        nodes = nodes,
                        currentNode = currentNode,
                        nodeStates = currentStates,
                        nodeRows = buildNodeRows(
                            chain = chain,
                            nodes = nodes,
                            currentNode = currentNode,
                            nodeStates = currentStates,
                            defaultNodeUrls = it.defaultNodeUrls,
                        ),
                    )
                }

                refreshNodeStatuses(chain, System.nanoTime())
            }
        }
    }

    private fun refreshNodeStatuses(chain: Chain, refreshNonce: Long) {
        refreshJob?.cancel()
        refreshJob = viewModelScope.launch {
            val nodes = state.value.nodes
            if (nodes.isEmpty()) {
                updateState { current ->
                    if (current.chain == chain && current.refreshNonce <= refreshNonce) {
                        current.copy(
                            isRefreshing = false,
                            refreshNonce = refreshNonce,
                        )
                    } else {
                        current
                    }
                }
                return@launch
            }

            val currentNode = currentNodeFor(chain, nodes, state.value.currentNode)
            val loadingStates = nodes.associate { it.url to NodeStatusState.Loading }
            updateState { current ->
                if (current.chain != chain) {
                    current
                } else {
                    current.copy(
                        currentNode = currentNode,
                        isRefreshing = true,
                        refreshNonce = refreshNonce,
                        nodeStates = loadingStates,
                        nodeRows = buildNodeRows(
                            chain = chain,
                            nodes = nodes,
                            currentNode = currentNode,
                            nodeStates = loadingStates,
                            defaultNodeUrls = current.defaultNodeUrls,
                        ),
                    )
                }
            }

            supervisorScope {
                nodes.forEach { node ->
                    launch {
                        val nodeState = withContext(Dispatchers.IO) {
                            nodeStatusClient.getNodeStatus(chain, node.url).toStatusState()
                        }
                        updateState { current ->
                            if (
                                current.chain != chain ||
                                current.refreshNonce != refreshNonce ||
                                current.nodes.none { currentNode -> currentNode.url == node.url }
                            ) {
                                current
                            } else {
                                val currentNodes = current.nodes
                                val refreshedCurrentNode = currentNodeFor(chain, currentNodes, current.currentNode)
                                val nodeStates = visibleNodeStates(
                                    currentNodes,
                                    current.nodeStates + (node.url to nodeState),
                                )
                                current.copy(
                                    currentNode = refreshedCurrentNode,
                                    nodeStates = nodeStates,
                                    nodeRows = buildNodeRows(
                                        chain = chain,
                                        nodes = currentNodes,
                                        currentNode = refreshedCurrentNode,
                                        nodeStates = nodeStates,
                                        defaultNodeUrls = current.defaultNodeUrls,
                                    ),
                                )
                            }
                        }
                    }
                }
            }

            updateState { current ->
                if (current.chain != chain || current.refreshNonce != refreshNonce) {
                    current
                } else {
                    val currentNodes = current.nodes
                    val refreshedCurrentNode = currentNodeFor(chain, currentNodes, current.currentNode)
                    current.copy(
                        currentNode = refreshedCurrentNode,
                        isRefreshing = false,
                        nodeRows = buildNodeRows(
                            chain = chain,
                            nodes = currentNodes,
                            currentNode = refreshedCurrentNode,
                            nodeStates = visibleNodeStates(currentNodes, current.nodeStates),
                            defaultNodeUrls = current.defaultNodeUrls,
                        ),
                    )
                }
            }
        }
    }

    private fun updateState(transform: (State) -> State) {
        state.update { current ->
            transform(current).also { updated ->
                _uiState.value = updated.toUIState()
            }
        }
    }

    private fun currentNodeFor(chain: Chain, nodes: List<Node>, selectedNode: Node? = null): Node {
        val selectedUrl = selectedNode?.url ?: getCurrentNodeCase.getCurrentNode(chain)?.url
        return nodes.firstOrNull { it.url == selectedUrl } ?: getGemNode(chain)
    }

    private data class State(
        val chain: Chain? = null,
        val explorers: List<String> = emptyList(),
        val currentNode: Node? = null,
        val currentExplorer: String? = null,
        val nodeStates: Map<String, NodeStatusState> = emptyMap(),
        val nodes: List<Node> = emptyList(),
        val nodeRows: List<NodeRowUiModel> = emptyList(),
        val availableChains: List<Chain> = emptyList(),
        val selectChain: Boolean = true,
        val availableAddNode: Boolean = true,
        val defaultNodeUrls: Set<String> = emptySet(),
        val isRefreshing: Boolean = false,
        val refreshNonce: Long = 0,
    ) {
        fun toUIState(): NetworksUIState {
            return NetworksUIState(
                chain = chain,
                chains = availableChains,
                selectChain = selectChain,
                blockExplorers = explorers,
                currentExplorer = currentExplorer,
                availableAddNode = availableAddNode,
                nodeRows = nodeRows,
                isRefreshing = isRefreshing,
            )
        }
    }
}

internal fun visibleNodeStates(
    nodes: List<Node>,
    nodeStates: Map<String, NodeStatusState>,
): Map<String, NodeStatusState> {
    val nodeUrls = nodes.mapTo(hashSetOf()) { it.url }
    return nodeStates.filterKeys(nodeUrls::contains)
}

internal fun buildNodeRows(
    chain: Chain,
    nodes: List<Node>,
    currentNode: Node,
    nodeStates: Map<String, NodeStatusState>,
    defaultNodeUrls: Set<String>,
): List<NodeRowUiModel> {
    val gemNodeUrls = getGemNodeUrls(chain)

    return nodes.map { node ->
        NodeRowUiModel(
            node = node,
            host = displayHost(node.url),
            gemNodeFlag = getGemNodeRegion(node.url)?.flag,
            selected = node.url == currentNode.url,
            canDelete = node.url !in gemNodeUrls && node.url !in defaultNodeUrls,
            statusState = nodeStates[node.url] ?: NodeStatusState.Loading,
        )
    }
}

internal fun NodeStatus?.toStatusState(): NodeStatusState = when {
    this == null || blockNumber == 0UL -> NodeStatusState.Error
    else -> NodeStatusState.Result(
        latestBlock = blockNumber,
        latency = latency,
        chainId = chainId,
    )
}

private fun displayHost(url: String): String {
    return runCatching { URI(url).host }
        .getOrNull()
        ?.takeIf { it.isNotBlank() }
        ?: url.removePrefix("https://").removePrefix("http://")
}
