package com.gemwallet.android.features.recipient.presents

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.WindowInsets
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
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.recipient.presents.components.RecipientHead
import com.gemwallet.android.features.recipient.presents.components.destinationView
import com.gemwallet.android.features.recipient.viewmodel.RecipientViewModel
import com.gemwallet.android.features.recipient.viewmodel.models.QrScanField
import com.gemwallet.android.features.recipient.viewmodel.models.RecipientHeadUIModel
import com.gemwallet.android.features.recipient.viewmodel.models.RecipientRowUIModel
import com.gemwallet.android.features.recipient.viewmodel.models.RecipientState
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.QrCodeScannerModal
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.fields.NameResolveIndicatorUIModel
import com.gemwallet.android.ui.components.isKeyboardVisible
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.listSections
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.ListSection
import com.gemwallet.android.ui.models.actions.AmountTransactionAction
import com.gemwallet.android.ui.models.actions.CancelAction
import com.gemwallet.android.ui.models.actions.ConfirmTransactionAction
import com.gemwallet.android.ui.theme.SceneSizing
import com.gemwallet.android.ui.theme.paddingDefault
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.QRScanType

@Composable
fun RecipientScreen(
    cancelAction: CancelAction,
    amountAction: AmountTransactionAction,
    confirmAction: ConfirmTransactionAction,
    viewModel: RecipientViewModel = hiltViewModel()
) {
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
        RecipientState.Loading -> Unit
        is RecipientState.Ready -> {
            RecipientScreen(
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

            QrCodeScannerModal(
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

@Composable
internal fun RecipientScreen(
    asset: Asset,
    head: RecipientHeadUIModel,
    hasMemo: Boolean,
    address: String,
    memo: String,
    addressError: Boolean,
    nameResolveIndicator: NameResolveIndicatorUIModel?,
    sections: List<ListSection<RecipientRowUIModel>>,
    buttonState: ButtonState,
    onAction: (RecipientAction) -> Unit,
) {
    val isKeyBoardOpen = WindowInsets.isKeyboardVisible
    val density = LocalDensity.current
    val isSmallScreen = with(density) {
        LocalWindowInfo.current.containerSize.height.toDp() < SceneSizing.compactContentHeight
    }

    Scene(
        title = stringResource(id = R.string.transfer_recipient_title),
        onClose = { onAction(RecipientAction.Cancel) },
        mainAction = {
            if (!isKeyBoardOpen || !isSmallScreen) {
                MainActionButton(
                    title = stringResource(id = R.string.common_continue),
                    state = buttonState,
                    onClick = { onAction(RecipientAction.Next) },
                )
            }
        },
        actions = {
            TextButton(onClick = { onAction(RecipientAction.Next) },
                enabled = buttonState == ButtonState.Enabled,
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
            item { RecipientHead(asset, head) }
            destinationView(
                hasMemo = hasMemo,
                assetName = asset.name,
                address = address,
                addressError = addressError,
                nameResolveIndicator = nameResolveIndicator,
                memo = memo,
                onAddress = { onAction(RecipientAction.SetAddress(it)) },
                onMemo = { onAction(RecipientAction.SetMemo(it)) },
                onQrScan = { onAction(RecipientAction.Scan(it)) },
                onSubmitAddress = { onAction(RecipientAction.ValidateAddress) },
            )
            listSections(sections) { position, item ->
                ListItem(
                    model = item.model,
                    listPosition = position,
                    modifier = Modifier.clickable {
                        item.memo?.let { onAction(RecipientAction.SetMemo(it)) }
                        onAction(RecipientAction.Select(item.recipient))
                    },
                    accessory = { DataBadgeChevron() },
                )
            }
        }
    }
}
