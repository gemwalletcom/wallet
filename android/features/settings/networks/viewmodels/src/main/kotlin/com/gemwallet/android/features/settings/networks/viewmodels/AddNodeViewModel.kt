package com.gemwallet.android.features.settings.networks.viewmodels

import uniffi.gemstone.GemAddNodeException
import uniffi.gemstone.GemChainSettingsServiceInterface
import uniffi.gemstone.GemNodeCheck
import kotlinx.coroutines.CancellationException
import androidx.compose.runtime.mutableStateOf
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.ui.R
import com.gemwallet.android.features.settings.networks.viewmodels.models.AddNodeUIModel
import com.wallet.core.primitives.Chain
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
    private val service: GemChainSettingsServiceInterface,
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
        try {
            val status = service.checkNode(chain.string, url)
            state.update { it.copy(nodeState = status, checking = false, errorResId = null) }
        } catch (error: GemAddNodeException.InvalidUrl) {
            state.update { it.copy(checking = false, errorResId = R.string.errors_invalid_url) }
        } catch (error: GemAddNodeException.InvalidNetworkId) {
            state.update { it.copy(checking = false, errorResId = R.string.errors_invalid_network_id) }
        } catch (error: CancellationException) {
            throw error
        } catch (_: Throwable) {
            state.update { it.copy(checking = false, errorResId = R.string.errors_error_occurred) }
        }
    }

    fun addUrl() {
        val chain = state.value.chain ?: return
        val status = state.value.nodeState ?: return
        viewModelScope.launch {
            if (runCatching { service.addNode(chain.string, status.url) }.isFailure) {
                state.update { it.copy(errorResId = R.string.errors_error_occurred) }
                return@launch
            }
            url.value = ""
            checkUrlJob?.cancel()
            state.update { State(chain = chain) }
        }
    }

    fun onUrlChange() {
        checkUrlJob?.cancel()
        val input = url.value.trim()
        state.update { it.copy(nodeState = null, checking = false, errorResId = null) }

        if (input.isEmpty()) {
            return
        }
        checkUrlJob = viewModelScope.launch {
            delay(service.nodeCheckDebounceMilliseconds().toLong())
            checkUrl(input)
        }
    }

    private data class State(
        val chain: Chain? = null,
        val nodeState: GemNodeCheck? = null,
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
