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
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.features.recipient.presents.components.RecipientHead
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
import com.wallet.core.primitives.NameRecord
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
                onAddress = viewModel::onAddress,
                onMemo = viewModel::onMemo,
                onQrScan = { scan = it },
                onNext = { viewModel.onNext(currentState.type, amountAction, confirmAction) },
                onDestination = { viewModel.onDestination(currentState.type, it, amountAction, confirmAction) },
                onCancel = cancelAction,
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
fun RecipientScreen(
    type: RecipientType,
    hasMemo: Boolean,
    address: String,
    memo: String,
    addressError: RecipientError,
    memoError: RecipientError,
    wallets: List<Wallet>,
    onAddress: (String, NameRecord?) -> Unit,
    onMemo: (String) -> Unit,
    onQrScan: (QrScanField) -> Unit,
    onNext: () -> Unit,
    onDestination: (DestinationAddress) -> Unit,
    onCancel: CancelAction,
) {
    val isKeyBoardOpen by keyboardAsState()
    val density = LocalDensity.current
    val isSmallScreen = with(density) {
        LocalWindowInfo.current.containerSize.height.toDp() < 680.dp
    }

    Scene(
        title = stringResource(id = R.string.transfer_recipient_title),
        onClose = { onCancel() },
        mainAction = {
            if (!isKeyBoardOpen || !isSmallScreen) {
                MainActionButton(
                    title = stringResource(id = R.string.common_continue),
                    onClick = onNext,
                )
            }
        },
        actions = {
            TextButton(onClick = onNext,
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
                onAddress = onAddress,
                onMemo = onMemo,
                onQrScan = onQrScan,
            )
            walletsDestination(toChain = type.assetInfo.asset.chain, items = wallets) { wallet, account ->
                onDestination(
                    DestinationAddress(
                        address = account.address,
                        name = wallet.name,
                    )
                )
            }
        }
    }
}
