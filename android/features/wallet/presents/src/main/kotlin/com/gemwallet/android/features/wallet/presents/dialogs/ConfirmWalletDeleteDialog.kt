package com.gemwallet.android.features.wallet.presents.dialogs

import androidx.compose.runtime.Composable
import androidx.compose.ui.platform.LocalContext
import com.gemwallet.android.model.AuthRequest
import com.gemwallet.android.ui.components.dialog.DeleteConfirmationDialog
import com.gemwallet.android.ui.requestAuth

@Composable
fun ConfirmWalletDeleteDialog(prompt: String, onConfirm: () -> Unit, onDismiss: () -> Unit) {
    val context = LocalContext.current
    DeleteConfirmationDialog(
        prompt = prompt,
        onConfirm = {
            onDismiss()
            context.requestAuth(AuthRequest.Confirmation, onSuccess = onConfirm)
        },
        onDismiss = onDismiss,
    )
}
