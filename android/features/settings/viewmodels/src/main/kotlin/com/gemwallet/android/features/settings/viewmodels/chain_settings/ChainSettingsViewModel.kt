package com.gemwallet.android.features.settings.viewmodels.chain_settings

import android.content.Context
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.features.settings.viewmodels.chain_settings.models.ChainSettingsUIState
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.models.navigation.requireChain
import com.wallet.core.primitives.Chain
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.supervisorScope
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemChainSettingsServiceInterface
import uniffi.gemstone.GemExplorerRow
import uniffi.gemstone.GemNodeListSession
import uniffi.gemstone.GemNodeStatusState
import javax.inject.Inject

@HiltViewModel
class ChainSettingsViewModel @Inject constructor(
    private val service: GemChainSettingsServiceInterface,
    savedStateHandle: SavedStateHandle,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    private val chain: Chain = savedStateHandle.requireChain()

    private val state = MutableStateFlow(State(explorers = service.explorerRows(chain.string), session = service.newNodeListSession(chain.string)))
    val uiState = state
        .map { it.toUIState() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, state.value.toUIState())

    private var observeNodesJob: Job? = null
    private var refreshJob: Job? = null

    init {
        observeNodes()
    }

    fun refresh() {
        observeNodes()
    }

    fun onSelectNode(url: String) {
        viewModelScope.launch {
            runCatchingCancellable { service.selectNode(chain.string, url) }
                .onSuccess { loadNodes() }
                .onFailure { error -> updateState { it.copy(errorText = error.errorText().text(context)) } }
        }
    }

    fun onSelectBlockExplorer(name: String) {
        runCatching { service.setExplorerName(chain.string, name) }
            .onSuccess { updateState { it.copy(explorers = service.explorerRows(chain.string)) } }
            .onFailure { error -> updateState { it.copy(errorText = error.errorText().text(context)) } }
    }

    fun onDeleteNode(url: String) {
        viewModelScope.launch {
            runCatchingCancellable { service.deleteNode(chain.string, url) }
                .onSuccess { loadNodes() }
                .onFailure { error -> updateState { it.copy(errorText = error.errorText().text(context)) } }
        }
    }

    fun clearError() = updateState { it.copy(errorText = null) }

    private fun observeNodes() {
        observeNodesJob?.cancel()
        observeNodesJob = viewModelScope.launch {
            loadNodes()
            refreshNodeStatuses()
        }
    }

    private suspend fun loadNodes() {
        runCatchingCancellable { service.nodes(chain.string) }
            .onSuccess { nodes -> updateState { it.copy(session = it.session.onNodes(nodes)) } }
            .onFailure { error -> updateState { it.copy(errorText = error.errorText().text(context)) } }
    }

    private fun refreshNodeStatuses() {
        refreshJob?.cancel()
        refreshJob = viewModelScope.launch {
            val urls = state.value.session.nodeUrls()
            if (urls.isEmpty()) {
                return@launch
            }
            updateState { it.copy(session = it.session.onChecking()) }

            supervisorScope {
                urls.forEach { url ->
                    launch {
                        val nodeState = withContext(ioDispatcher) { service.nodeStatus(chain.string, url) }
                        updateState { it.copy(session = it.session.onStatus(url, nodeState)) }
                    }
                }
            }
        }
    }

    private fun updateState(transform: (State) -> State) {
        state.update(transform)
    }

    private data class State(val explorers: List<GemExplorerRow>, val session: GemNodeListSession, val errorText: String? = null)

    private fun State.toUIState(): ChainSettingsUIState = ChainSettingsUIState(
        chain = chain,
        sections = session.sections(explorers),
        errorText = errorText,
    )
}
