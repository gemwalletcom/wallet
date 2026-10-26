package com.gemwallet.android.ui.components.fields

import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.size
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.icons.AppIcons

private val IconButtonSize = 36.dp

@Composable
fun TransferTextFieldActions(value: String, paste: (() -> Unit)? = null, qrScanner: (() -> Unit)? = null, onClean: () -> Unit) {
    if (value.isNotEmpty()) {
        IconButton(modifier = Modifier.size(IconButtonSize), onClick = onClean) {
            Icon(
                imageVector = AppIcons.Close,
                contentDescription = stringResource(R.string.filter_clear),
                tint = MaterialTheme.colorScheme.onSurface,
            )
        }
        return
    }
    Row {
        if (paste != null) {
            IconButton(modifier = Modifier.size(IconButtonSize), onClick = paste) {
                Icon(
                    imageVector = AppIcons.ContentPaste,
                    contentDescription = stringResource(R.string.common_paste),
                    tint = MaterialTheme.colorScheme.onSurface,
                )
            }
        }
        if (qrScanner != null) {
            IconButton(modifier = Modifier.size(IconButtonSize), onClick = qrScanner) {
                Icon(
                    imageVector = AppIcons.QrCodeScanner,
                    contentDescription = stringResource(R.string.wallet_scan_qr_code),
                    tint = MaterialTheme.colorScheme.onSurface,
                )
            }
        }
    }
}
