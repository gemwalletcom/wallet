package com.gemwallet.android.features.settings.viewmodels.chain_settings

import android.content.Context
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.runtime.snapshotFlow
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.features.settings.viewmodels.chain_settings.models.ChainSettingsUIState
import com.gemwallet.android.ui.localization.text
import com.wallet.core.primitives.Chain
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.supervisorScope
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemChainRow
import uniffi.gemstone.GemChainServiceInterface
import uniffi.gemstone.GemChainSettingsServiceInterface
import uniffi.gemstone.GemExplorerRow
import uniffi.gemstone.GemNodeListSession
import uniffi.gemstone.GemNodeStatusState
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class ChainSettingsViewModel @Inject constructor(
    private val service: GemChainSettingsServiceInterface,
    private val chainService: GemChainServiceInterface,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
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
            updateState { it.copy(availableChains = chainService.chainRows(null, "")) }
            snapshotFlow { chainFilter.text }.collectLatest { query ->
                updateState { it.copy(availableChains = chainService.chainRows(null, query.toString())) }
            }
        }
    }

    fun onSelectedChain(chain: Chain) {
        updateState {
            it.copy(
                chain = chain,
                selectChain = false,
                explorers = service.explorerRows(chain.string),
                session = service.newNodeListSession(chain.string),
            )
        }
        observeNodes(chain)
    }

    fun refresh() {
        val chain = state.value.chain ?: return
        observeNodes(chain)
    }

    fun onSelectNode(url: String) {
        val chain = state.value.chain ?: return
        viewModelScope.launch {
            runCatchingCancellable { service.selectNode(chain.string, url) }
                .onSuccess { loadNodes(chain) }
                .onFailure { error -> updateState { it.copy(errorText = error.errorText().text(context)) } }
        }
    }

    fun onSelectBlockExplorer(name: String) {
        val chain = state.value.chain ?: return
        runCatching { service.setExplorerName(chain.string, name) }
            .onSuccess { updateState { it.copy(explorers = service.explorerRows(chain.string)) } }
            .onFailure { error -> updateState { it.copy(errorText = error.errorText().text(context)) } }
    }

    fun onSelectChain() {
        updateState { it.copy(selectChain = true) }
    }

    fun onDeleteNode(url: String) {
        val chain = state.value.chain ?: return
        viewModelScope.launch {
            runCatchingCancellable { service.deleteNode(chain.string, url) }
                .onSuccess { loadNodes(chain) }
                .onFailure { error -> updateState { it.copy(errorText = error.errorText().text(context)) } }
        }
    }

    fun clearError() = updateState { it.copy(errorText = null) }

    private fun observeNodes(chain: Chain) {
        observeNodesJob?.cancel()
        observeNodesJob = viewModelScope.launch {
            loadNodes(chain)
            refreshNodeStatuses(chain)
        }
    }

    private suspend fun loadNodes(chain: Chain) {
        runCatchingCancellable { service.nodes(chain.string) }
            .onSuccess { nodes -> updateState { it.copy(session = it.session?.onNodes(nodes)) } }
            .onFailure { error -> updateState { it.copy(errorText = error.errorText().text(context)) } }
    }

    private fun refreshNodeStatuses(chain: Chain) {
        refreshJob?.cancel()
        refreshJob = viewModelScope.launch {
            val urls = state.value.session?.nodeUrls().orEmpty()
            if (urls.isEmpty()) {
                return@launch
            }
            updateState { current -> if (current.chain == chain) current.copy(session = current.session?.onChecking()) else current }

            supervisorScope {
                urls.forEach { url ->
                    launch {
                        val nodeState = withContext(ioDispatcher) { service.nodeStatus(chain.string, url) }
                        updateState { current ->
                            if (current.chain != chain) current else current.copy(session = current.session?.onStatus(url, nodeState))
                        }
                    }
                }
            }
        }
    }

    private fun updateState(transform: (State) -> State) {
        state.update(transform)
    }

    private data class State(
        val chain: Chain? = null,
        val explorers: List<GemExplorerRow> = emptyList(),
        val session: GemNodeListSession? = null,
        val availableChains: List<GemChainRow> = emptyList(),
        val selectChain: Boolean = true,
        val errorText: String? = null,
    )

    private fun State.toUIState(): ChainSettingsUIState = ChainSettingsUIState(
        chain = chain,
        chains = availableChains,
        selectChain = selectChain,
        sections = session?.sections(explorers).orEmpty(),
        errorText = errorText,
    )
}
