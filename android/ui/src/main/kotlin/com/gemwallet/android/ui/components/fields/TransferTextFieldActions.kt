package com.gemwallet.android.ui.components.fields

import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Clear
import androidx.compose.material.icons.filled.ContentPaste
import androidx.compose.material.icons.filled.QrCodeScanner
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

private val IconButtonSize = 36.dp

@Composable
fun TransferTextFieldActions(
    value: String,
    paste: (() -> Unit)? = null,
    qrScanner: (() -> Unit)? = null,
    onClean: () -> Unit
) {
    if (value.isNotEmpty()) {
        IconButton(modifier = Modifier.size(IconButtonSize), onClick = onClean) {
            Icon(
                imageVector = Icons.Default.Clear,
                contentDescription = "clear",
                tint = MaterialTheme.colorScheme.onSurface,
            )
        }
        return
    }
    Row {
        if (paste != null) {
            IconButton(modifier = Modifier.size(IconButtonSize), onClick = paste) {
                Icon(
                    imageVector = Icons.Default.ContentPaste,
                    contentDescription = "paste",
                    tint = MaterialTheme.colorScheme.onSurface,
                )
            }
        }
        if (qrScanner != null) {
            IconButton(modifier = Modifier.size(IconButtonSize), onClick = qrScanner) {
                Icon(
                    imageVector = Icons.Default.QrCodeScanner,
                    contentDescription = "scan_address",
                    tint = MaterialTheme.colorScheme.onSurface,
                )
            }
        }
    }
}