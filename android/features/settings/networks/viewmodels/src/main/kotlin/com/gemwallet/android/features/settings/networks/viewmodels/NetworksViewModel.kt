package com.gemwallet.android.features.settings.networks.viewmodels

import com.gemwallet.android.ext.requireChain
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.runtime.snapshotFlow
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import uniffi.gemstone.GemChainSettingsServiceInterface
import uniffi.gemstone.GemNodeSelection
import uniffi.gemstone.GemNodeStatusState
import com.gemwallet.android.features.settings.networks.viewmodels.models.NodeRowUiModel
import com.gemwallet.android.features.settings.networks.viewmodels.models.NetworksUIState
import com.wallet.core.primitives.Chain
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
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class NetworksViewModel @Inject constructor(
    private val service: GemChainSettingsServiceInterface,
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
            updateState { it.copy(availableChains = service.chains("").map { it.requireChain() }) }
            snapshotFlow { chainFilter.text }.collectLatest { query ->
                updateState { it.copy(availableChains = service.chains(query.toString()).map { it.requireChain() }) }
            }
        }
    }

    fun onSelectedChain(chain: Chain) {
        updateState {
            it.copy(
                chain = chain,
                selectChain = false,
                explorers = service.explorers(chain.string),
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

    fun onSelectNode(url: String) {
        val chain = state.value.chain ?: return
        viewModelScope.launch {
            service.selectNode(chain.string, url)
            loadNodes(chain)
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

    fun onDeleteNode(url: String) {
        val chain = state.value.chain ?: return
        viewModelScope.launch {
            service.deleteNode(chain.string, url)
            loadNodes(chain)
        }
    }

    private fun observeNodes(chain: Chain) {
        observeNodesJob?.cancel()
        observeNodesJob = viewModelScope.launch {
            loadNodes(chain)
            refreshNodeStatuses(chain, System.nanoTime())
        }
    }

    private suspend fun loadNodes(chain: Chain) {
        val nodes = buildNodeRows(
            selections = service.nodes(chain.string),
            gemNodeFlag = service::nodeFlag,
            canDelete = { url -> canDeleteNode(chain, url) },
        )

        updateState {
            it.copy(
                nodes = nodes,
                nodeStates = visibleNodeStates(nodes, it.nodeStates),
            )
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

            val loadingStates = nodes.associate { it.id to GemNodeStatusState.Loading }
            updateState { current ->
                if (current.chain != chain) {
                    current
                } else {
                    current.copy(
                        refreshNonce = refreshNonce,
                        nodeStates = loadingStates,
                    )
                }
            }

            supervisorScope {
                nodes.forEach { node ->
                    launch {
                        val nodeState = withContext(Dispatchers.IO) {
                            service.nodeStatus(chain.string, node.id)
                        }
                        updateNodesIfCurrent(chain, refreshNonce) { current ->
                            if (current.nodes.none { it.id == node.id }) {
                                current
                            } else {
                                current.copy(
                                    nodeStates = visibleNodeStates(
                                        current.nodes,
                                        current.nodeStates + (node.id to nodeState),
                                    ),
                                )
                            }
                        }
                    }
                }
            }
        }
    }

    private fun updateNodesIfCurrent(chain: Chain, refreshNonce: Long, transform: (State) -> State) {
        updateState { current ->
            if (current.chain != chain || current.refreshNonce != refreshNonce) current
            else transform(current)
        }
    }

    private fun updateState(transform: (State) -> State) {
        state.update(transform)
    }

    private fun canDeleteNode(chain: Chain, url: String): Boolean = service.canDeleteNode(chain.string, url)

    private data class State(
        val chain: Chain? = null,
        val explorers: List<String> = emptyList(),
        val currentExplorer: String? = null,
        val nodeStates: Map<String, GemNodeStatusState> = emptyMap(),
        val nodes: List<NodeRowUiModel> = emptyList(),
        val availableChains: List<Chain> = emptyList(),
        val selectChain: Boolean = true,
        val availableAddNode: Boolean = true,
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
                nodeRows = nodes.map { it.copy(statusState = nodeStates[it.id] ?: GemNodeStatusState.Loading) },
            )
        }
    }
}

internal fun visibleNodeStates(
    nodes: List<NodeRowUiModel>,
    nodeStates: Map<String, GemNodeStatusState>,
): Map<String, GemNodeStatusState> {
    val nodeUrls = nodes.mapTo(hashSetOf()) { it.id }
    return nodeStates.filterKeys(nodeUrls::contains)
}

internal fun buildNodeRows(
    selections: List<GemNodeSelection>,
    gemNodeFlag: (String) -> String?,
    canDelete: (String) -> Boolean,
): List<NodeRowUiModel> {
    return selections.map { selection ->
        NodeRowUiModel(
            url = selection.url,
            host = selection.host,
            gemNodeFlag = gemNodeFlag(selection.url),
            selected = selection.isSelected,
            canDelete = canDelete(selection.url),
        )
    }
}
