package com.gemwallet.android.features.settings.viewmodels.chain_settings

import androidx.compose.runtime.mutableStateOf
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.ext.GemConstants
import com.wallet.core.primitives.Chain
import dagger.assisted.Assisted
import dagger.assisted.AssistedFactory
import dagger.assisted.AssistedInject
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemAddNodeException
import uniffi.gemstone.GemAddNodeSession
import uniffi.gemstone.GemAddNodeViewState
import uniffi.gemstone.GemChainSettingsServiceInterface
import uniffi.gemstone.GemServiceException

@HiltViewModel(assistedFactory = AddNodeViewModel.Factory::class)
class AddNodeViewModel @AssistedInject constructor(@Assisted private val chain: Chain, private val service: GemChainSettingsServiceInterface, @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher) : ViewModel() {

    private val session = MutableStateFlow(service.newAddNodeSession(chain.string))
    val viewState: StateFlow<GemAddNodeViewState> = session.map { it.viewState() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, session.value.viewState())
    val url = mutableStateOf("")
    val isAdded = MutableStateFlow(false)
    private var checkUrlJob: Job? = null
    private var addUrlJob: Job? = null

    fun clear() {
        checkUrlJob?.cancel()
        url.value = ""
        session.value = service.newAddNodeSession(chain.string)
    }

    fun onUrlChange() {
        checkUrlJob?.cancel()
        val current = session.value.onInput(url.value)
        session.value = current
        if (!current.checksUrl()) {
            return
        }
        checkUrlJob = viewModelScope.launch {
            delay(GemConstants.nodeCheckDebounce)
            checkUrl(current)
        }
    }

    fun addUrl() {
        if (addUrlJob?.isActive == true) return
        val current = session.value
        val status = current.viewState().canImport.takeIf { it }?.let { (current.check) } ?: return
        addUrlJob = viewModelScope.launch {
            try {
                withContext(ioDispatcher) { service.addNode(current.chain, status.url) }
            } catch (error: GemServiceException) {
                if (session.value == current) session.value = current.onAddFailed(error)
                return@launch
            } catch (error: CancellationException) {
                throw error
            } catch (_: Throwable) {
                if (session.value == current) session.value = current.onAddFailed(null)
                return@launch
            }
            if (session.value == current) {
                url.value = ""
                checkUrlJob?.cancel()
                session.value = current.onImported()
            }
            isAdded.value = true
        }
    }

    private suspend fun checkUrl(current: GemAddNodeSession) {
        session.value = current.onChecking()
        session.value = try {
            current.onChecked(current.url, withContext(ioDispatcher) { service.checkNode(current.chain, current.url) })
        } catch (error: GemAddNodeException) {
            current.onCheckFailed(current.url, error)
        } catch (error: CancellationException) {
            throw error
        } catch (_: Throwable) {
            current.onCheckFailed(current.url, null)
        }
    }

    @AssistedFactory
    interface Factory {
        fun create(chain: Chain): AddNodeViewModel
    }
}
