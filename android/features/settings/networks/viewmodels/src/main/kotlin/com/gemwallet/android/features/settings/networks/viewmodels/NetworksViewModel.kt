package com.gemwallet.android.features.settings.networks.viewmodels

import uniffi.gemstone.GemExplorerService
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.runtime.snapshotFlow
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.blockchain.services.NodeStatusService
import com.gemwallet.android.cases.nodes.DeleteNodeCase
import com.gemwallet.android.cases.nodes.GetCurrentNodeCase
import com.gemwallet.android.cases.nodes.GetNodesCase
import com.gemwallet.android.cases.nodes.SetCurrentNodeCase
import com.gemwallet.android.cases.nodes.getGemNode
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
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
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
    private val explorerService: GemExplorerService,
    private val getCurrentNodeCase: GetCurrentNodeCase,
    private val setCurrentNodeCase: SetCurrentNodeCase,
    private val deleteNodeCase: DeleteNodeCase,
    private val nodeStatusClient: NodeStatusService,
    private val config: Config,
) : ViewModel() {

    private val state = MutableStateFlow(State())
    val uiState = state
        .map { it.toUIState() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, state.value.toUIState())
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
        val defaultNodeUrls = getNodesCase.getDefaultNodes(chain).mapTo(linkedSetOf()) { it.url }
        val gemNodeFlags = config.getNodeRegions().associate { region ->
            config.getNodeUrl(chain.string, region) to config.getNodeRegionFlag(region)
        }

        updateState {
            it.copy(
                chain = chain,
                selectChain = false,
                explorers = explorerService.getExplorers(chain.string),
                currentNode = getCurrentNodeCase.getCurrentNode(chain),
                currentExplorer = explorerService.getExplorerName(chain.string),
                availableAddNode = true,
                defaultNodeUrls = defaultNodeUrls,
                gemNodeFlags = gemNodeFlags,
                nodes = emptyList(),
                nodeStates = emptyMap(),
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
        updateState { it.copy(currentNode = node) }
    }

    fun onSelectBlockExplorer(name: String) {
        val chain = state.value.chain ?: return
        explorerService.setExplorerName(chain.string, name)
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
                        refreshNonce = refreshNonce,
                        nodeStates = loadingStates,
                    )
                }
            }

            supervisorScope {
                nodes.forEach { node ->
                    launch {
                        val nodeState = withContext(Dispatchers.IO) {
                            nodeStatusClient.getNodeStatus(chain, node.url).toStatusState()
                        }
                        updateNodesIfCurrent(chain, refreshNonce) { current ->
                            if (current.nodes.none { it.url == node.url }) {
                                current
                            } else {
                                val nodeStates = visibleNodeStates(
                                    current.nodes,
                                    current.nodeStates + (node.url to nodeState),
                                )
                                current.copy(nodeStates = nodeStates).withCurrentNodeFor(chain)
                            }
                        }
                    }
                }
            }

            updateNodesIfCurrent(chain, refreshNonce) { current ->
                current.withCurrentNodeFor(chain)
            }
        }
    }

    private fun updateNodesIfCurrent(chain: Chain, refreshNonce: Long, transform: (State) -> State) {
        updateState { current ->
            if (current.chain != chain || current.refreshNonce != refreshNonce) current
            else transform(current)
        }
    }

    private fun State.withCurrentNodeFor(chain: Chain): State =
        copy(currentNode = currentNodeFor(chain, nodes, currentNode))

    private fun updateState(transform: (State) -> State) {
        state.update(transform)
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
        val availableChains: List<Chain> = emptyList(),
        val selectChain: Boolean = true,
        val availableAddNode: Boolean = true,
        val defaultNodeUrls: Set<String> = emptySet(),
        val gemNodeFlags: Map<String, String> = emptyMap(),
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
                nodeRows = if (chain == null || currentNode == null) {
                    emptyList()
                } else {
                    buildNodeRows(
                        nodes = nodes,
                        currentNode = currentNode,
                        nodeStates = nodeStates,
                        defaultNodeUrls = defaultNodeUrls,
                        gemNodeFlags = gemNodeFlags,
                    )
                },
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
    nodes: List<Node>,
    currentNode: Node,
    nodeStates: Map<String, NodeStatusState>,
    defaultNodeUrls: Set<String>,
    gemNodeFlags: Map<String, String>,
): List<NodeRowUiModel> {
    return nodes.map { node ->
        NodeRowUiModel(
            node = node,
            host = displayHost(node.url),
            gemNodeFlag = gemNodeFlags[node.url],
            selected = node.url == currentNode.url,
            canDelete = node.url !in gemNodeFlags && node.url !in defaultNodeUrls,
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
