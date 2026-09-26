package com.gemwallet.android.features.wallet_connector.viewmodels

import android.content.Context
import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.wallet_connect.WalletConnectPendingRequest
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.features.wallet_connector.viewmodels.models.ReviewTexts
import com.gemwallet.android.features.wallet_connector.viewmodels.models.SignMessageUIState
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.buttonState
import dagger.assisted.Assisted
import dagger.assisted.AssistedFactory
import dagger.assisted.AssistedInject
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import uniffi.gemstone.GemServiceException
import uniffi.gemstone.GemSignMessageServiceInterface
import uniffi.gemstone.GemSignerFailure
import uniffi.gemstone.GemWalletConnectServiceInterface
import uniffi.gemstone.applicationConnectionRow
import uniffi.gemstone.signerFailure

@HiltViewModel(assistedFactory = SignMessageViewModel.Factory::class)
class SignMessageViewModel @AssistedInject constructor(
    @Assisted private val request: WalletConnectPendingRequest.SignMessage,
    private val service: GemWalletConnectServiceInterface,
    signMessageService: GemSignMessageServiceInterface,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    private val isSigning = MutableStateFlow(false)

    private val preview = SignMessageUIState(request, applicationConnectionRow(request.appMetadata.toGem()), signMessageService, ReviewTexts(context), context)

    val uiState: StateFlow<SignMessageUIState> = flow { emit(preview.withAddressNames()) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, preview)

    val buttonState: StateFlow<ButtonState> = combine(uiState, isSigning) { state, signing ->
        buttonState(enabled = !state.hasCriticalWarning, loading = signing)
    }.stateIn(viewModelScope, SharingStarted.Eagerly, ButtonState.Enabled)

    fun onSign(onSigned: (String) -> Unit, onError: (String) -> Unit) {
        if (isSigning.value) {
            return
        }
        isSigning.value = true
        val state = uiState.value
        viewModelScope.launch(ioDispatcher) {
            val signature = try {
                service.signMessage(state.wallet.id.id, state.signMessage)
            } catch (err: GemServiceException) {
                Log.e(TAG, "Sign message failed topic=${request.sessionId}", err)
                isSigning.value = false
                when (val failure = signerFailure(err.text())) {
                    is GemSignerFailure.Retry -> onError(failure.error.text(context))
                    GemSignerFailure.Reject -> request.reject()
                }
                return@launch
            }
            onSigned(signature)
        }
    }

    fun onReject(reject: () -> Unit) {
        if (isSigning.value) {
            return
        }
        reject()
    }

    @AssistedFactory
    interface Factory {
        fun create(request: WalletConnectPendingRequest.SignMessage): SignMessageViewModel
    }

    private companion object {
        const val TAG = "WalletConnect"
    }
}
