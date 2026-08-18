package com.gemwallet.android.ui.components.fields

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
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.platform.LocalClipboard
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import com.gemwallet.android.ui.components.GemTextField
import com.gemwallet.android.ui.components.clipboard.getPlainText
import com.gemwallet.android.ui.components.progress.CircularProgressIndicator16
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.name.NameRecordState
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingHalfSmall
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.sceneContentPadding
import com.gemwallet.android.ui.theme.smallIconSize

@Composable
fun ColumnScope.AddressChainField(
    value: String,
    label: String,
    onValueChange: (String) -> Unit,
    state: NameRecordState = NameRecordState.None,
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
                NameResolveIndicator(state)
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
