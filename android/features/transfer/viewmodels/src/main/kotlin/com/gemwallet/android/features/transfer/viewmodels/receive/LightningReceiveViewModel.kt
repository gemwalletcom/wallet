package com.gemwallet.android.features.transfer.viewmodels.receive

import android.content.Context
import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.connection.cases.ObserveRefreshInterval
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.ext.GemConstants
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.tickerFlow
import com.gemwallet.android.math.numberFormat
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.models.ToastMessage
import com.gemwallet.android.ui.models.ToastPresenter
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.FlowPreview
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.debounce
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemLightningReceive
import uniffi.gemstone.GemLightningReceiveState
import uniffi.gemstone.GemLightningServiceInterface
import uniffi.gemstone.GemRefreshKind
import uniffi.gemstone.GemServiceException
import javax.inject.Inject
import kotlin.time.Duration

@OptIn(ExperimentalCoroutinesApi::class, FlowPreview::class)
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

    private val amountFlow = MutableStateFlow("")
    val amount: StateFlow<String> = amountFlow.asStateFlow()

    private val refreshEnabled = MutableStateFlow(false)

    init {
        viewModelScope.launch {
            amountFlow.debounce { if (it.isEmpty()) Duration.ZERO else GemConstants.lightningInvoiceDebounce }.collectLatest(::receive)
        }
        viewModelScope.launch {
            combine(refreshEnabled, stateFlow.map { it is State.Ready }.distinctUntilChanged(), observeRefreshInterval.refreshIntervalMillis(GemRefreshKind.RECEIVE)) { isEnabled, isReady, interval ->
                interval.takeIf { isEnabled && isReady }
            }
                .distinctUntilChanged()
                .flatMapLatest { interval -> interval?.let { tickerFlow(it) {} } ?: emptyFlow() }
                .collect { checkPayment() }
        }
    }

    fun onAmount(value: String) {
        amountFlow.value = value
    }

    fun onRetry() {
        viewModelScope.launch { receive(amountFlow.value) }
    }

    fun setRefreshEnabled(isEnabled: Boolean) {
        refreshEnabled.value = isEnabled
    }

    private suspend fun receive(amount: String) {
        val walletId = walletId ?: return run { stateFlow.value = State.Unavailable }
        runCatchingCancellable { withContext(ioDispatcher) { service.receive(walletId, amount, numberFormat()) } }
            .onSuccess { stateFlow.value = State.Ready(it) }
            .onFailure(::onFailure)
    }

    private suspend fun checkPayment() {
        val walletId = walletId ?: return
        val receive = (stateFlow.value as? State.Ready)?.receive ?: return
        runCatchingCancellable { withContext(ioDispatcher) { service.receiveState(walletId, receive.copy.value) } }
            .onSuccess { state ->
                if (stateFlow.value != State.Ready(receive)) return
                when (state) {
                    GemLightningReceiveState.PENDING -> Unit
                    GemLightningReceiveState.EXPIRED -> receive(amountFlow.value)
                    GemLightningReceiveState.RECEIVED -> onReceived()
                }
            }
            .onFailure { error -> if (error is GemServiceException.Cancelled) stateFlow.value = State.Locked else Log.e(TAG, "lightning payment check failed", error) }
    }

    private fun onFailure(error: Throwable) {
        if (error is GemServiceException.Cancelled) {
            stateFlow.value = State.Locked
            return
        }
        Log.e(TAG, "lightning receive failed", error)
        stateFlow.value = State.Unavailable
    }

    private suspend fun onReceived() {
        toastPresenter.emitToast(ToastMessage(context.getString(R.string.transaction_title_received), R.drawable.ic_check_circle))
        stateFlow.value = State.Received
    }

    private companion object {
        const val TAG = "LightningReceive"
    }
}
