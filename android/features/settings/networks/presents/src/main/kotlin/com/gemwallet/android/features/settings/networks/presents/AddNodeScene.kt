package com.gemwallet.android.features.settings.networks.presents

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
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
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.ext.asset
import com.gemwallet.android.features.settings.networks.viewmodels.AddNodeViewModel
import com.gemwallet.android.features.settings.networks.viewmodels.models.NodeCheckRowUIModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.GemTextField
import com.gemwallet.android.ui.components.QrCodeScannerModal
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.clipboard.clipboardManager
import com.gemwallet.android.ui.components.clipboard.getPlainText
import com.gemwallet.android.ui.components.fields.TransferTextFieldActions
import com.gemwallet.android.ui.components.list_item.AssetListItem
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.Spacer16
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.QRScanType

@Composable
fun AddNodeScene(chain: Chain, onCancel: () -> Unit) {
    val viewModel: AddNodeViewModel = hiltViewModel()
    val uiModel by viewModel.uiModel.collectAsStateWithLifecycle()

    DisposableEffect(chain) {
        viewModel.init(chain)
        onDispose { }
    }

    var isShowQRScan by remember { mutableStateOf(false) }

    BackHandler {
        onCancel()
    }

    Scene(
        title = stringResource(id = R.string.nodes_import_node_title),
        mainAction = {
            MainActionButton(
                title = stringResource(id = R.string.wallet_import_action),
                state = uiModel.buttonState,
            ) {
                viewModel.addUrl()
                onCancel()
            }
        },
        onClose = onCancel,
    ) {
        val asset = chain.asset()
        AssetListItem(
            asset = asset,
            listPosition = ListPosition.Single,
        )
        UrlField(
            value = viewModel.url,
            error = uiModel.errorText,
            onValueChange = viewModel::onUrlChange,
            onQRScan = {
                isShowQRScan = true
            }
        )
        Spacer16()
        uiModel.checks.forEach { NodeCheckRow(it) }
        uiModel.warning?.let { ListItem(model = it, listPosition = ListPosition.Single) }
    }

    QrCodeScannerModal(
        isVisible = isShowQRScan,
        scanType = QRScanType.Url,
        onDismissRequest = { isShowQRScan = false },
        onResult = {
            isShowQRScan = false
            viewModel.url.value = it.trim()
            viewModel.onUrlChange()
        },
    )
}

@Composable
private fun UrlField(
    value: MutableState<String> = mutableStateOf(""),
    error: String = "",
    onValueChange: () -> Unit,
    onQRScan: () -> Unit,
) {
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
                qrScanner = onQRScan
            )
        }
    )
}

@Composable
private fun NodeCheckRow(row: NodeCheckRowUIModel) {
    val isInSync = row.isInSync
    ListItem(
        model = row.model,
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
