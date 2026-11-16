package com.gemwallet.android.ui.components.dialog

import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R

@Composable
fun ErrorDialog(message: String?, onDismiss: () -> Unit, title: String? = null) {
    message ?: return

    AlertDialog(
        onDismissRequest = onDismiss,
        containerColor = MaterialTheme.colorScheme.background,
        confirmButton = {
            Button(onDismiss) {
                Text(stringResource(R.string.common_done))
            }
        },
        title = title?.let { { Text(it) } },
        text = { Text(message) },
    )
}
