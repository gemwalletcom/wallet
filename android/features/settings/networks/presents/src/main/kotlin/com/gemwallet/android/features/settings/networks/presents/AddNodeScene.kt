package com.gemwallet.android.features.settings.networks.presents

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Cancel
import androidx.compose.material.icons.filled.CheckCircleOutline
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
import androidx.compose.ui.platform.LocalClipboard
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.ext.asset
import com.gemwallet.android.features.settings.networks.viewmodels.AddNodeViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.GemTextField
import com.gemwallet.android.ui.components.QrCodeScannerModal
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.clipboard.getPlainText
import com.gemwallet.android.ui.components.fields.TransferTextFieldActions
import com.gemwallet.android.ui.components.list_item.AssetListItem
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemTitleText
import com.gemwallet.android.ui.components.list_item.property.PropertyDataText
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyTitleText
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.Spacer16
import com.wallet.core.primitives.Chain
import java.text.NumberFormat

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
                enabled = uiModel.canImport,
                loading = uiModel.checking,
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
            error = uiModel.errorResId?.let { stringResource(it) }.orEmpty(),
            onValueChange = viewModel::onUrlChange,
            onQRScan = {
                isShowQRScan = true
            }
        )
        Spacer16()
        if (uiModel.canImport) {
            val nf = NumberFormat.getInstance()
            val status = requireNotNull(uiModel.status)

            PropertyItem(R.string.nodes_import_node_chain_id, status.chainId)
            PropertyItem(
                title = {
                    PropertyTitleText(R.string.nodes_import_node_in_sync)
                },
                data = {
                    PropertyDataText(
                        "",
                        badge = {
                            if (status.inSync) {
                                Icon(
                                    imageVector = Icons.Default.CheckCircleOutline,
                                    tint = MaterialTheme.colorScheme.tertiary,
                                    contentDescription = ""
                                )
                            } else {
                                Icon(
                                    imageVector = Icons.Default.Cancel,
                                    tint = MaterialTheme.colorScheme.error,
                                    contentDescription = ""
                                )
                            }
                        }
                    )
                },
            )
            PropertyItem(R.string.nodes_import_node_latest_block, nf.format(status.blockNumber.toLong()))
            PropertyItem(
                R.string.nodes_import_node_latency,
                stringResource(R.string.common_latency_in_ms, status.latency.toLong())
            )
            WarningItem()
        }
    }

    QrCodeScannerModal(
        isVisible = isShowQRScan,
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
    val clipboardManager = LocalClipboard.current.nativeClipboard
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
private fun WarningItem() {
    ListItem(
        listPosition = ListPosition.Single,
        title = {
            ListItemTitleText(stringResource(R.string.asset_verification_warning_title))
        },
        subtitle = {
            Text(
                text = stringResource(R.string.nodes_import_node_warning_message),
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.secondary,
            )
        },
    )
}
