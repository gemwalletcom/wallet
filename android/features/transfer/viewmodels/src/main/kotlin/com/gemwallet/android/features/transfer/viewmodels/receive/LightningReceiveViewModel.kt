package com.gemwallet.android.features.transfer.viewmodels.receive

import android.content.Context
import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.connection.cases.ObserveRefreshInterval
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.models.ToastMessage
import com.gemwallet.android.ui.models.ToastPresenter
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemLightningReceive
import uniffi.gemstone.GemLightningReceiveState
import uniffi.gemstone.GemLightningServiceInterface
import uniffi.gemstone.GemRefreshKind
import uniffi.gemstone.GemServiceException
import javax.inject.Inject

@HiltViewModel
class LightningReceiveViewModel @Inject constructor(
    private val service: GemLightningServiceInterface,
    getSession: GetSession,
    observeRefreshInterval: ObserveRefreshInterval,
    private val toastPresenter: ToastPresenter,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    sealed interface State {
        data object Loading : State
        data object Locked : State
        data object Unavailable : State
        data class Ready(val receive: GemLightningReceive) : State
        data object Received : State
    }

    private val walletId = getSession().value?.wallet?.id?.id

    private val stateFlow = MutableStateFlow<State>(State.Loading)
    val state: StateFlow<State> = stateFlow.asStateFlow()

    val refreshIntervalMillis: StateFlow<Long> = observeRefreshInterval.refreshIntervalMillis(GemRefreshKind.RECEIVE)
        .stateIn(viewModelScope, SharingStarted.Eagerly, 0L)

    private var checkJob: Job? = null

    init {
        viewModelScope.launch { receive() }
    }

    fun onRetry() {
        viewModelScope.launch { receive() }
    }

    fun refresh() {
        if (checkJob?.isActive == true) return
        checkJob = viewModelScope.launch { checkPayment() }
    }

    private suspend fun receive() {
        val walletId = walletId ?: return run { stateFlow.value = State.Unavailable }
        runCatchingCancellable { withContext(ioDispatcher) { service.receive(walletId) } }
            .onSuccess { stateFlow.value = State.Ready(it) }
            .onFailure { error ->
                if (error !is GemServiceException.Cancelled) Log.e(TAG, "lightning receive failed", error)
                stateFlow.value = if (error is GemServiceException.Cancelled) State.Locked else State.Unavailable
            }
    }

    private suspend fun checkPayment() {
        val walletId = walletId ?: return
        val receive = (stateFlow.value as? State.Ready)?.receive ?: return
        runCatchingCancellable { withContext(ioDispatcher) { service.receiveState(walletId, receive.copy.value) } }
            .onSuccess { state ->
                if (stateFlow.value != State.Ready(receive)) return
                when (state) {
                    GemLightningReceiveState.PENDING -> Unit
                    GemLightningReceiveState.EXPIRED -> receive()
                    GemLightningReceiveState.RECEIVED -> {
                        toastPresenter.emitToast(ToastMessage(context.getString(R.string.transaction_title_received), R.drawable.ic_check_circle))
                        stateFlow.value = State.Received
                    }
                }
            }
            .onFailure { error -> if (error is GemServiceException.Cancelled) stateFlow.value = State.Locked else Log.e(TAG, "lightning payment check failed", error) }
    }

    private companion object {
        const val TAG = "LightningReceive"
    }
}
