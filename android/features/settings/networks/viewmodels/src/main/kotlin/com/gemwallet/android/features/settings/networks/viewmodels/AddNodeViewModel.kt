package com.gemwallet.android.features.settings.networks.viewmodels

import android.content.Context
import dagger.hilt.android.qualifiers.ApplicationContext
import com.gemwallet.android.features.settings.networks.viewmodels.models.uiModel
import uniffi.gemstone.GemAddNodeException
import uniffi.gemstone.GemAddNodeFailure
import uniffi.gemstone.GemAddNodeSession
import uniffi.gemstone.GemChainSettingsServiceInterface
import kotlinx.coroutines.withContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CancellationException
import androidx.compose.runtime.mutableStateOf
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.features.settings.networks.viewmodels.models.AddNodeUIModel
import com.gemwallet.android.data.services.gemstone.di.IoDispatcher
import com.wallet.core.primitives.Chain
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject

@HiltViewModel
class AddNodeViewModel @Inject constructor(
    private val service: GemChainSettingsServiceInterface,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    private val session = MutableStateFlow<GemAddNodeSession?>(null)
    val uiModel = session.map { it?.uiModel(context) ?: AddNodeUIModel() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, AddNodeUIModel())
    val url = mutableStateOf("")
    private var checkUrlJob: Job? = null

    fun init(chain: Chain) {
        checkUrlJob?.cancel()
        url.value = ""
        session.value = service.newAddNodeSession(chain.string)
    }

    fun onUrlChange() {
        checkUrlJob?.cancel()
        val current = session.value?.onInput(url.value) ?: return
        session.value = current
        if (!current.checksUrl()) {
            return
        }
        checkUrlJob = viewModelScope.launch {
            delay(service.nodeCheckDebounceMilliseconds().toLong())
            checkUrl(current)
        }
    }

    fun addUrl() {
        val current = session.value ?: return
        val status = current.viewState().canImport.takeIf { it }?.let { (current.check) } ?: return
        viewModelScope.launch {
            if (runCatching { withContext(ioDispatcher) { service.addNode(current.chain, status.url) } }.isFailure) {
                session.value = current.onFailed(GemAddNodeFailure.UNAVAILABLE)
                return@launch
            }
            url.value = ""
            checkUrlJob?.cancel()
            session.value = current.onImported()
        }
    }

    private suspend fun checkUrl(current: GemAddNodeSession) {
        session.value = current.onChecking()
        session.value = try {
            current.onChecked(withContext(ioDispatcher) { service.checkNode(current.chain, current.url) })
        } catch (error: GemAddNodeException.InvalidUrl) {
            current.onFailed(GemAddNodeFailure.INVALID_URL)
        } catch (error: GemAddNodeException.InvalidNetworkId) {
            current.onFailed(GemAddNodeFailure.INVALID_NETWORK_ID)
        } catch (error: CancellationException) {
            throw error
        } catch (_: Throwable) {
            current.onFailed(GemAddNodeFailure.UNAVAILABLE)
        }
    }
}
