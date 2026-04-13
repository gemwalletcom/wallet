package com.gemwallet.android.features.recipient.presents.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.platform.LocalClipboard
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import com.gemwallet.android.features.recipient.viewmodel.models.RecipientError
import com.gemwallet.android.ui.components.GemTextField
import com.gemwallet.android.ui.components.clipboard.getPlainText
import com.gemwallet.android.ui.components.fields.TransferTextFieldActions
import com.gemwallet.android.ui.theme.paddingHalfSmall

@Composable
fun MemoTextField(
    value: String,
    label: String,
    onValueChange: (String) -> Unit,
    error: RecipientError = RecipientError.None,
    onQrScanner: (() -> Unit)? = null,
) {
    val keyboardController = LocalSoftwareKeyboardController.current
    val clipboardManager = LocalClipboard.current.nativeClipboard
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
        if (error != RecipientError.None) {
            Text(
                modifier = Modifier.fillMaxWidth(),
                text = recipientErrorString(error),
                color = MaterialTheme.colorScheme.error,
                style = MaterialTheme.typography.labelMedium,
            )
        }
    }
}