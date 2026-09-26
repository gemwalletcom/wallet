package com.gemwallet.android.features.transfer.presents.recipient

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.qr_scanner.presents.QRScannerModal
import com.gemwallet.android.features.transfer.viewmodels.recipient.RecipientViewModel
import com.gemwallet.android.features.transfer.viewmodels.recipient.models.QrScanField
import com.gemwallet.android.features.transfer.viewmodels.recipient.models.RecipientUIState
import com.gemwallet.android.ui.models.actions.AmountTransactionAction
import com.gemwallet.android.ui.models.actions.CancelAction
import com.gemwallet.android.ui.models.actions.ConfirmTransactionAction
import com.wallet.core.primitives.QRScanType

@Composable
fun RecipientScreen(cancelAction: CancelAction, amountAction: AmountTransactionAction, confirmAction: ConfirmTransactionAction, viewModel: RecipientViewModel = hiltViewModel()) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    val hasMemo by viewModel.hasMemo.collectAsStateWithLifecycle()
    val sections by viewModel.sections.collectAsStateWithLifecycle()
    val addressError by viewModel.addressError.collectAsStateWithLifecycle()
    val address by viewModel.address.collectAsStateWithLifecycle()
    val buttonState by viewModel.buttonState.collectAsStateWithLifecycle()
    val memo by viewModel.memo.collectAsStateWithLifecycle()
    val nameResolveIndicator by viewModel.nameResolveIndicator.collectAsStateWithLifecycle()

    var scan by remember { mutableStateOf(QrScanField.None) }

    when (val currentState = state) {
        RecipientUIState.Loading -> Unit

        is RecipientUIState.Ready -> {
            RecipientScene(
                asset = currentState.asset,
                head = currentState.head,
                hasMemo = hasMemo,
                address = address,
                memo = memo,
                addressError = addressError,
                nameResolveIndicator = nameResolveIndicator,
                sections = sections,
                buttonState = buttonState,
                onAction = { action ->
                    when (action) {
                        is RecipientAction.SetAddress -> viewModel.onAddress(action.address)
                        is RecipientAction.SetMemo -> viewModel.onMemo(action.memo)
                        is RecipientAction.Scan -> scan = action.field
                        RecipientAction.Next -> viewModel.onNext(currentState, amountAction, confirmAction)
                        RecipientAction.ValidateAddress -> viewModel.onValidateAddress()
                        is RecipientAction.Select -> viewModel.onDestination(currentState, action.destination, amountAction, confirmAction)
                        RecipientAction.Cancel -> cancelAction()
                    }
                },
            )

            QRScannerModal(
                isVisible = scan != QrScanField.None,
                scanType = when (scan) {
                    QrScanField.Memo -> QRScanType.Memo
                    else -> QRScanType.Address
                },
                onDismissRequest = { scan = QrScanField.None },
                onResult = {
                    viewModel.setQrData(currentState, scan, it, confirmAction)
                    scan = QrScanField.None
                },
            )
        }
    }
}
