package com.gemwallet.android

import androidx.compose.material3.AlertDialog
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R

@Composable
internal fun ErrorDialog(
    error: String?,
    onDismiss: () -> Unit,
) {
    if (error.isNullOrEmpty()) return

    AlertDialog(
        onDismissRequest = onDismiss,
        containerColor = MaterialTheme.colorScheme.background,
        confirmButton = {
            TextButton(onClick = onDismiss) {
                Text(text = stringResource(id = R.string.common_done))
            }
        },
        text = { Text(text = error) },
    )
}
