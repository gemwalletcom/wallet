package com.gemwallet.android.features.recipient.presents

import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalWindowInfo
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.cases.contacts.ContactRecipient
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.features.recipient.presents.components.RecipientHead
import com.gemwallet.android.features.recipient.presents.components.contactsDestination
import com.gemwallet.android.features.recipient.presents.components.destinationView
import com.gemwallet.android.features.recipient.presents.components.walletsDestination
import com.gemwallet.android.features.recipient.viewmodel.RecipientViewModel
import com.gemwallet.android.features.recipient.viewmodel.models.QrScanField
import com.gemwallet.android.features.recipient.viewmodel.models.RecipientError
import com.gemwallet.android.features.recipient.viewmodel.models.RecipientState
import com.gemwallet.android.features.recipient.viewmodel.models.RecipientType
import com.gemwallet.android.model.DestinationAddress
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.QrCodeScannerModal
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.keyboardAsState
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.models.actions.AmountTransactionAction
import com.gemwallet.android.ui.models.actions.CancelAction
import com.gemwallet.android.ui.models.actions.ConfirmTransactionAction
import com.gemwallet.android.ui.theme.paddingDefault
import com.wallet.core.primitives.Wallet

@Composable
fun RecipientScreen(
    cancelAction: CancelAction,
    amountAction: AmountTransactionAction,
    confirmAction: ConfirmTransactionAction,
    viewModel: RecipientViewModel = hiltViewModel()
) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    val hasMemo by viewModel.hasMemo.collectAsStateWithLifecycle()
    val wallets by viewModel.wallets.collectAsStateWithLifecycle()
    val contacts by viewModel.contacts.collectAsStateWithLifecycle()
    val addressError by viewModel.addressError.collectAsStateWithLifecycle()
    val memoError by viewModel.memoErrorState.collectAsStateWithLifecycle()
    val address by viewModel.address.collectAsStateWithLifecycle()
    val memo by viewModel.memo.collectAsStateWithLifecycle()

    var scan by remember { mutableStateOf(QrScanField.None) }

    when (val currentState = state) {
        RecipientState.Loading -> Unit
        is RecipientState.Ready -> {
            RecipientScreen(
                type = currentState.type,
                hasMemo = hasMemo,
                address = address,
                memo = memo,
                addressError = addressError,
                memoError = memoError,
                wallets = wallets,
                contacts = contacts,
                onAction = { action ->
                    when (action) {
                        is RecipientAction.SetAddress -> viewModel.onAddress(action.address, action.nameRecord)
                        is RecipientAction.SetMemo -> viewModel.onMemo(action.memo)
                        is RecipientAction.Scan -> scan = action.field
                        RecipientAction.Next -> viewModel.onNext(currentState.type, amountAction, confirmAction)
                        is RecipientAction.Select -> viewModel.onDestination(currentState.type, action.destination, amountAction, confirmAction)
                        RecipientAction.Cancel -> cancelAction()
                    }
                },
            )

            QrCodeScannerModal(
                isVisible = scan != QrScanField.None,
                onDismissRequest = { scan = QrScanField.None },
                onResult = {
                    viewModel.setQrData(currentState.type, scan, it, confirmAction)
                    scan = QrScanField.None
                },
            )
        }
    }
}

@Composable
internal fun RecipientScreen(
    type: RecipientType,
    hasMemo: Boolean,
    address: String,
    memo: String,
    addressError: RecipientError,
    memoError: RecipientError,
    wallets: List<Wallet>,
    contacts: List<ContactRecipient>,
    onAction: (RecipientAction) -> Unit,
) {
    val isKeyBoardOpen by keyboardAsState()
    val density = LocalDensity.current
    val isSmallScreen = with(density) {
        LocalWindowInfo.current.containerSize.height.toDp() < 680.dp
    }

    Scene(
        title = stringResource(id = R.string.transfer_recipient_title),
        onClose = { onAction(RecipientAction.Cancel) },
        mainAction = {
            if (!isKeyBoardOpen || !isSmallScreen) {
                MainActionButton(
                    title = stringResource(id = R.string.common_continue),
                    onClick = { onAction(RecipientAction.Next) },
                )
            }
        },
        actions = {
            TextButton(onClick = { onAction(RecipientAction.Next) },
                colors = ButtonDefaults.textButtonColors()
                    .copy(contentColor = MaterialTheme.colorScheme.primary)
            ) {
                Text(stringResource(R.string.common_continue).uppercase())
            }
        }
    ) {
        LazyColumn(
            contentPadding = PaddingValues(bottom = paddingDefault),
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            item { RecipientHead(type) }
            destinationView(
                asset = type.assetInfo,
                hasMemo = hasMemo,
                address = address,
                addressError = addressError,
                memo = memo,
                memoError = memoError,
                onAddress = { input, nameRecord -> onAction(RecipientAction.SetAddress(input, nameRecord)) },
                onMemo = { onAction(RecipientAction.SetMemo(it)) },
                onQrScan = { onAction(RecipientAction.Scan(it)) },
            )
            contactsDestination(contacts = contacts) { contact ->
                onAction(RecipientAction.SetMemo(contact.memo ?: ""))
                onAction(
                    RecipientAction.Select(
                        DestinationAddress(
                            address = contact.address,
                            name = contact.name,
                        )
                    )
                )
            }
            walletsDestination(toChain = type.assetInfo.asset.chain, items = wallets) { wallet, account ->
                onAction(
                    RecipientAction.Select(
                        DestinationAddress(
                            address = account.address,
                            name = wallet.name,
                        )
                    )
                )
            }
        }
    }
}
