package com.gemwallet.android.features.settings.networks.viewmodels

import androidx.compose.runtime.mutableStateOf
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.blockchain.services.NodeStatusService
import com.gemwallet.android.cases.nodes.AddNodeCase
import com.gemwallet.android.cases.nodes.SetCurrentNodeCase
import com.gemwallet.android.ext.getNetworkId
import com.gemwallet.android.model.NodeStatus
import com.gemwallet.android.ui.R
import com.gemwallet.android.features.settings.networks.viewmodels.models.AddNodeUIModel
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Node
import com.wallet.core.primitives.NodeState
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import javax.inject.Inject

@HiltViewModel
class AddNodeViewModel @Inject constructor(
    private val nodeStatusClient: NodeStatusService,
    private val addNodeCase: AddNodeCase,
    private val setCurrentNodeCase: SetCurrentNodeCase,
) : ViewModel() {

    private val state = MutableStateFlow(State())
    val uiModel = state.map { it.toUIModel() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, AddNodeUIModel())
    val url = mutableStateOf("")
    private var checkUrlJob: Job? = null

    fun init(chain: Chain) {
        checkUrlJob?.cancel()
        url.value = ""
        state.update { State(chain = chain) }
    }

    private suspend fun checkUrl(url: String) {
        state.update { it.copy(checking = true, nodeState = null, errorResId = null) }
        val chain = state.value.chain ?: return
        val status = nodeStatusClient.getNodeStatus(chain, url)
        when {
            status == null -> state.update {
                it.copy(
                    checking = false,
                    errorResId = R.string.errors_error_occured,
                )
            }

            status.chainId != chain.getNetworkId() -> state.update {
                it.copy(
                    checking = false,
                    errorResId = R.string.errors_invalid_network_id,
                )
            }

            else -> state.update {
                it.copy(
                    nodeState = status,
                    checking = false,
                    errorResId = null,
                )
            }
        }
    }

    fun addUrl() {
        val chain = state.value.chain ?: return
        val status = state.value.nodeState ?: return
        viewModelScope.launch {
            val addResult = runCatching { addNodeCase.addNode(chain = chain, url.value) }
            if (addResult.isFailure) {
                state.update { it.copy(errorResId = R.string.errors_error_occured) }
                return@launch
            }

            setCurrentNodeCase.setCurrentNode(chain = chain, Node(status.url, status = NodeState.Active, 0))
            url.value = ""
            checkUrlJob?.cancel()
            state.update { State(chain = chain) }
        }
    }

    fun onUrlChange() {
        checkUrlJob?.cancel()
        state.update { it.copy(nodeState = null, checking = false, errorResId = null) }

        if (url.value.isValidNodeUrl()) {
            val currentUrl = url.value
            checkUrlJob = viewModelScope.launch {
                delay(500)
                if (currentUrl == url.value) {
                    checkUrl(currentUrl)
                }
            }
        }
    }

    private data class State(
        val chain: Chain? = null,
        val nodeState: NodeStatus? = null,
        val checking: Boolean = false,
        val errorResId: Int? = null,
    ) {
        fun toUIModel(): AddNodeUIModel {
            return AddNodeUIModel(
                chain = chain,
                status = nodeState,
                checking = checking,
                errorResId = errorResId,
            )
        }
    }
}
