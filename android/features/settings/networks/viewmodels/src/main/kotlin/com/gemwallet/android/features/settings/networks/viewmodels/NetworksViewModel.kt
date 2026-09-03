package com.gemwallet.android.features.settings.networks.viewmodels

import com.gemwallet.android.ext.toChain
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.runtime.snapshotFlow
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import uniffi.gemstone.GemChainSettingsServiceInterface
import uniffi.gemstone.GemNodeStatusState
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.features.settings.networks.viewmodels.models.NodeRowUiModel
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

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class NetworksViewModel @Inject constructor(
    private val service: GemChainSettingsServiceInterface,
) : ViewModel() {

    private val state = MutableStateFlow(State())
    val uiState = state
        .map { it.toUIState(::canDeleteNode, service::nodeFlag) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, state.value.toUIState(::canDeleteNode, service::nodeFlag))
    val chainFilter = TextFieldState()

    private var observeNodesJob: Job? = null
    private var refreshJob: Job? = null

    init {
        viewModelScope.launch {
            updateState { it.copy(availableChains = service.chains("").mapNotNull { it.toChain() }) }
            snapshotFlow { chainFilter.text }.collectLatest { query ->
                updateState { it.copy(availableChains = service.chains(query.toString()).mapNotNull { it.toChain() }) }
            }
        }
    }

    fun onSelectedChain(chain: Chain) {
        updateState {
            it.copy(
                chain = chain,
                selectChain = false,
                explorers = service.explorers(chain.string),
                currentNode = service.selectedNode(chain.string).decodeJson(),
                currentExplorer = service.explorerName(chain.string),
                availableAddNode = true,
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
        viewModelScope.launch {
            service.selectNode(chain.string, node.url)
            updateState { it.copy(currentNode = node) }
        }
    }

    fun onSelectBlockExplorer(name: String) {
        val chain = state.value.chain ?: return
        service.setExplorerName(chain.string, name)
        updateState { it.copy(currentExplorer = name) }
    }

    fun onSelectChain() {
        updateState { it.copy(selectChain = true) }
    }

    fun onDeleteNode(node: Node) {
        val chain = state.value.chain ?: return
        viewModelScope.launch {
            service.deleteNode(chain.string, node.url)
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
            val nodes = service.nodes(chain.string).map { it.decodeJson<Node>() }
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
            val loadingStates = nodes.associate { it.url to GemNodeStatusState.Loading }
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
                            service.nodeStatus(chain.string, node.url)
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

    private fun canDeleteNode(chain: Chain, url: String): Boolean = service.canDeleteNode(chain.string, url)

    private fun currentNodeFor(chain: Chain, nodes: List<Node>, selectedNode: Node? = null): Node {
        val current = service.selectedNode(chain.string).decodeJson<Node>()
        return nodes.firstOrNull { it.url == (selectedNode?.url ?: current.url) } ?: current
    }

    private data class State(
        val chain: Chain? = null,
        val explorers: List<String> = emptyList(),
        val currentNode: Node? = null,
        val currentExplorer: String? = null,
        val nodeStates: Map<String, GemNodeStatusState> = emptyMap(),
        val nodes: List<Node> = emptyList(),
        val availableChains: List<Chain> = emptyList(),
        val selectChain: Boolean = true,
        val availableAddNode: Boolean = true,
        val refreshNonce: Long = 0,
    ) {
        fun toUIState(canDeleteNode: (Chain, String) -> Boolean, gemNodeFlag: (String) -> String?): NetworksUIState {
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
                        gemNodeFlag = gemNodeFlag,
                        canDelete = { url -> canDeleteNode(chain, url) },
                    )
                },
            )
        }
    }
}

internal fun visibleNodeStates(
    nodes: List<Node>,
    nodeStates: Map<String, GemNodeStatusState>,
): Map<String, GemNodeStatusState> {
    val nodeUrls = nodes.mapTo(hashSetOf()) { it.url }
    return nodeStates.filterKeys(nodeUrls::contains)
}

internal fun buildNodeRows(
    nodes: List<Node>,
    currentNode: Node,
    nodeStates: Map<String, GemNodeStatusState>,
    gemNodeFlag: (String) -> String?,
    canDelete: (String) -> Boolean,
): List<NodeRowUiModel> {
    return nodes.map { node ->
        NodeRowUiModel(
            node = node,
            host = displayHost(node.url),
            gemNodeFlag = gemNodeFlag(node.url),
            selected = node.url == currentNode.url,
            canDelete = canDelete(node.url),
            statusState = nodeStates[node.url] ?: GemNodeStatusState.Loading,
        )
    }
}

private fun displayHost(url: String): String {
    return runCatching { URI(url).host }
        .getOrNull()
        ?.takeIf { it.isNotBlank() }
        ?: url.removePrefix("https://").removePrefix("http://")
}
