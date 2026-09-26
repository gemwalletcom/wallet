package com.gemwallet.android.features.assets.viewmodels.select

import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.domains.confirm.ConfirmTransferInput
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.ui.models.actions.AmountTransactionAction
import com.gemwallet.android.ui.models.actions.ConfirmTransactionAction
import com.wallet.core.primitives.AssetId
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.launch
import uniffi.gemstone.GemPaymentRecipient
import uniffi.gemstone.GemPaymentServiceInterface
import uniffi.gemstone.GemPaymentStep
import javax.inject.Inject

@HiltViewModel
class SendPaymentViewModel @Inject constructor(private val paymentService: GemPaymentServiceInterface) : ViewModel() {

    fun onSelect(assetId: AssetId, payment: GemPaymentRecipient?, recipientAction: (AssetId, GemPaymentRecipient?) -> Unit, amountAction: AmountTransactionAction, confirmAction: ConfirmTransactionAction) {
        if (payment == null) {
            recipientAction(assetId, null)
            return
        }
        viewModelScope.launch {
            runCatchingCancellable { paymentService.prepareAsset(payment, assetId.toIdentifier()) }
                .onSuccess { step ->
                    when (step) {
                        is GemPaymentStep.Confirm -> confirmAction(ConfirmTransferInput(step.transfer))
                        is GemPaymentStep.Amount -> amountAction(AmountParams.Transfer(assetId, step.payment))
                        is GemPaymentStep.Recipient -> recipientAction(assetId, step.payment)
                    }
                }
                .onFailure { Log.e(TAG, "preparing the payment for ${assetId.toIdentifier()} failed", it) }
        }
    }
}

private const val TAG = "SendPayment"
