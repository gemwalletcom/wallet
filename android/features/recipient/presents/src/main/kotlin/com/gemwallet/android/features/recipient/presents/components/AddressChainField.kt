package com.gemwallet.android.features.recipient.presents.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.platform.LocalClipboard
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.recipient.viewmodel.AddressChainViewModel
import com.gemwallet.android.features.recipient.viewmodel.NameRecordState
import com.gemwallet.android.ui.components.GemTextField
import com.gemwallet.android.ui.components.clipboard.getPlainText
import com.gemwallet.android.ui.components.fields.TransferTextFieldActions
import com.gemwallet.android.ui.components.progress.CircularProgressIndicator16
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingHalfSmall
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.sceneContentPadding
import com.gemwallet.android.ui.theme.smallIconSize
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.NameRecord

@Composable
fun ColumnScope.AddressChainField(
    value: String,
    label: String,
    state: NameRecordState,
    onValueChange: (String) -> Unit,
    error: String = "",
    editable: Boolean = true,
    onPaste: ((String) -> Unit)? = null,
    onQrScanner: (() -> Unit)? = null,
) {
    val keyboardController = LocalSoftwareKeyboardController.current
    val clipboardManager = LocalClipboard.current.nativeClipboard

    GemTextField(
        modifier = Modifier
            .fillMaxWidth()
            .onFocusChanged {
                if (it.hasFocus) keyboardController?.show() else keyboardController?.hide()
            },
        value = value,
        singleLine = true,
        readOnly = !editable,
        label = label,
        onValueChange = onValueChange,
        trailing = {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(paddingSmall),
            ) {
                when (state) {
                    NameRecordState.Loading -> CircularProgressIndicator16()
                    NameRecordState.Error -> Icon(
                        modifier = Modifier.size(smallIconSize),
                        imageVector = AppIcons.Error,
                        contentDescription = null,
                        tint = MaterialTheme.colorScheme.error,
                    )
                    is NameRecordState.Complete -> Icon(
                        modifier = Modifier.size(smallIconSize),
                        imageVector = AppIcons.CheckCircle,
                        contentDescription = null,
                        tint = MaterialTheme.colorScheme.tertiary,
                    )
                    NameRecordState.None -> Unit
                }
                TransferTextFieldActions(
                    value = value,
                    paste = { (onPaste ?: onValueChange)(clipboardManager.getPlainText() ?: "") },
                    qrScanner = onQrScanner,
                    onClean = { onValueChange("") },
                )
            }
        }
    )
    if (error.isNotEmpty()) {
        Text(
            modifier = Modifier
                .fillMaxWidth()
                .padding(start = sceneContentPadding() + paddingDefault, end = sceneContentPadding(), top = paddingHalfSmall),
            text = error,
            color = MaterialTheme.colorScheme.error,
            style = MaterialTheme.typography.bodySmall,
        )
    }
}

@Composable
fun ColumnScope.AddressChainField(
    chain: Chain?,
    value: String,
    label: String,
    onValueChange: (String, NameRecord?) -> Unit,
    error: String = "",
    editable: Boolean = true,
    searchName: Boolean = true,
    onQrScanner: (() -> Unit)? = null,
    viewModel: AddressChainViewModel = hiltViewModel(),
) {
    val uiState by viewModel.uiState.collectAsStateWithLifecycle()

    LaunchedEffect(key1 = value) {
        viewModel.onNameRecord(chain, value)
    }

    LaunchedEffect(key1 = uiState.nameRecord?.address) {
        onValueChange(uiState.nameRecord?.name ?: value, uiState.nameRecord)
    }

    AddressChainField(
        value = value,
        label = label,
        state = uiState,
        onValueChange = { newValue ->
            if (searchName) {
                viewModel.onInput(newValue, chain)
            }
            onValueChange(newValue, uiState.nameRecord)
        },
        error = error,
        editable = editable,
        onQrScanner = onQrScanner,
    )
}
