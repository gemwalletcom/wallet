package com.gemwallet.android.ui.components.fields

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import com.gemwallet.android.ui.components.GemTextField
import com.gemwallet.android.ui.components.clipboard.getPlainText
import com.gemwallet.android.ui.theme.paddingHalfSmall
import com.gemwallet.android.ui.components.clipboard.clipboardManager

@Composable
fun MemoTextField(
    value: String,
    label: String,
    onValueChange: (String) -> Unit,
    error: String = "",
    onQrScanner: (() -> Unit)? = null,
) {
    val keyboardController = LocalSoftwareKeyboardController.current
    val clipboardManager = LocalContext.current.clipboardManager()
    Column(
        modifier = Modifier,
        verticalArrangement = Arrangement.spacedBy(paddingHalfSmall),
    ) {
        GemTextField(
            modifier = Modifier
                .fillMaxWidth()
                .onFocusChanged {
                    if (it.hasFocus) keyboardController?.show() else keyboardController?.hide()
                },
            value = value,
            singleLine = true,
            label = label,
            onValueChange = onValueChange,
            trailing = {
                TransferTextFieldActions(
                    value = value,
                    paste = { onValueChange(clipboardManager.getPlainText() ?: "") },
                    qrScanner = onQrScanner,
                    onClean = {
                        onValueChange("")
                    }
                )
            }
        )
        if (error.isNotEmpty()) {
            Text(
                modifier = Modifier.fillMaxWidth(),
                text = error,
                color = MaterialTheme.colorScheme.error,
                style = MaterialTheme.typography.labelMedium,
            )
        }
    }
}
