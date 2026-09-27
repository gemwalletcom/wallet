package com.gemwallet.android.features.settings.presents.chain_settings

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.MutableState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import com.gemwallet.android.features.qr_scanner.presents.QRScannerModal
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.GemTextField
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.clipboard.clipboardManager
import com.gemwallet.android.ui.components.clipboard.getPlainText
import com.gemwallet.android.ui.components.fields.TransferTextFieldActions
import com.gemwallet.android.ui.components.list_item.ChainItem
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.localization.stringRes
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.buttonState
import com.gemwallet.android.ui.theme.Spacer16
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.QRScanType
import uniffi.gemstone.GemAddNodePhase
import uniffi.gemstone.GemAddNodeViewState
import uniffi.gemstone.GemNodeCheckRow
import uniffi.gemstone.GemNodeSyncState
import uniffi.gemstone.chainRow

@Composable
fun AddNodeScene(chain: Chain, state: GemAddNodeViewState?, url: MutableState<String>, onUrlChange: () -> Unit, onAdd: () -> Unit, onCancel: () -> Unit) {
    val context = LocalContext.current
    var isShowQRScan by remember { mutableStateOf(false) }
    val phase = state?.phase

    BackHandler {
        onCancel()
    }

    Scene(
        title = stringResource(id = R.string.nodes_import_node_title),
        mainAction = {
            MainActionButton(
                title = stringResource(id = R.string.wallet_import_action),
                state = buttonState(enabled = state?.canImport == true, loading = phase is GemAddNodePhase.Checking),
            ) {
                onAdd()
            }
        },
        onClose = onCancel,
    ) {
        ChainItem(
            row = chainRow(chain.string),
            listPosition = ListPosition.Single,
            onClick = null,
        )
        UrlField(
            value = url,
            error = (phase as? GemAddNodePhase.Failed)?.error?.text(context).orEmpty(),
            onValueChange = onUrlChange,
            onQRScan = {
                isShowQRScan = true
            },
        )
        Spacer16()
        (phase as? GemAddNodePhase.Ready)?.rows.orEmpty().forEach { NodeCheckRow(it) }
        if (state?.showsWarning == true) {
            ListItem(
                model = ListItemModel(title = stringResource(R.string.asset_verification_warning_title), titleExtra = stringResource(R.string.nodes_import_node_warning_message)),
                listPosition = ListPosition.Single,
            )
        }
    }

    QRScannerModal(
        isVisible = isShowQRScan,
        scanType = QRScanType.Url,
        onDismissRequest = { isShowQRScan = false },
        onResult = {
            isShowQRScan = false
            url.value = it.trim()
            onUrlChange()
        },
    )
}

@Composable
private fun UrlField(value: MutableState<String> = mutableStateOf(""), error: String = "", onValueChange: () -> Unit, onQRScan: () -> Unit) {
    val keyboardController = LocalSoftwareKeyboardController.current
    val clipboardManager = LocalContext.current.clipboardManager()
    GemTextField(
        modifier = Modifier
            .fillMaxWidth()
            .onFocusChanged {
                if (it.hasFocus) keyboardController?.show() else keyboardController?.hide()
            },
        value = value.value,
        singleLine = true,
        label = stringResource(R.string.common_url),
        error = error,
        onValueChange = { newValue ->
            value.value = newValue
            onValueChange()
        },
        trailing = {
            TransferTextFieldActions(
                value = value.value,
                paste = {
                    value.value = clipboardManager.getPlainText()?.trim().orEmpty()
                    onValueChange()
                },
                onClean = {
                    value.value = ""
                    onValueChange()
                },
                qrScanner = onQRScan,
            )
        },
    )
}

@Composable
private fun NodeCheckRow(row: GemNodeCheckRow) {
    val context = LocalContext.current
    val isInSync = (row as? GemNodeCheckRow.InSync)?.state?.let { it == GemNodeSyncState.IN_SYNC }
    ListItem(
        model = ListItemModel(title = stringResource(row.stringRes()), subtitle = row.text(context).takeIf { row !is GemNodeCheckRow.InSync }),
        listPosition = ListPosition.Middle,
        accessory = isInSync?.let {
            {
                Icon(
                    imageVector = if (it) AppIcons.CheckCircleOutlined else AppIcons.Cancel,
                    tint = if (it) MaterialTheme.colorScheme.tertiary else MaterialTheme.colorScheme.error,
                    contentDescription = "",
                )
            }
        },
    )
}
