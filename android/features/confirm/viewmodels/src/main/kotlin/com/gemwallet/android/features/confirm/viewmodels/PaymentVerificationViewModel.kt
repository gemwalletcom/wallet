package com.gemwallet.android.features.confirm.viewmodels

import android.content.Context
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.domains.confirm.ConfirmTransferInput
import com.gemwallet.android.domains.wallet.chainAddresses
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.models.ToastEmitter
import com.gemwallet.android.ui.models.ToastEmitterImpl
import com.gemwallet.android.ui.models.ToastMessage
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import uniffi.gemstone.GemPaymentException
import uniffi.gemstone.GemPaymentLoad
import uniffi.gemstone.GemPaymentServiceInterface
import javax.inject.Inject

@HiltViewModel
class PaymentVerificationViewModel @Inject constructor(
    private val getSession: GetSession,
    private val paymentService: GemPaymentServiceInterface,
    savedStateHandle: SavedStateHandle,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel(),
    ToastEmitter by ToastEmitterImpl() {

    private val link = savedStateHandle.requirePaymentLink()
    private val urlState = MutableStateFlow(savedStateHandle.requireUrl())
    private val confirmState = MutableStateFlow<ConfirmTransferInput?>(null)

    val url: StateFlow<String> = urlState.asStateFlow()
    val confirm: StateFlow<ConfirmTransferInput?> = confirmState.asStateFlow()
    val verificationBridge = PaymentVerificationBridge(::onPaymentVerified)
    private var verifying: Job? = null

    private fun onPaymentVerified() {
        if (verifying?.isActive == true) return
        verifying = viewModelScope.launch(ioDispatcher) {
            val wallet = getSession().value?.wallet ?: return@launch
            try {
                when (val load = paymentService.load(link, wallet.chainAddresses.map { it.toGem() })) {
                    is GemPaymentLoad.Sign -> confirmState.value = ConfirmTransferInput(load.transfer)
                    is GemPaymentLoad.Verify -> urlState.value = load.url
                }
            } catch (error: CancellationException) {
                throw error
            } catch (error: GemPaymentException) {
                emitToast(ToastMessage(error.errorText().text(context), R.drawable.ic_warning))
            }
        }
    }
}
