package com.gemwallet.android

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import com.gemwallet.android.ui.components.RootWarningDialog
import com.gemwallet.android.ui.components.isDeviceRooted
import com.gemwallet.android.ui.theme.WalletTheme
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

@Composable
internal fun RootWarningHost(
    onCancel: () -> Unit,
) {
    WalletTheme {
        var showRootWarningDialog by remember { mutableStateOf(false) }

        LaunchedEffect(Unit) {
            showRootWarningDialog = withContext(Dispatchers.Default) {
                isDeviceRooted()
            }
        }

        if (showRootWarningDialog) {
            RootWarningDialog(
                onCancel = onCancel,
                onIgnore = { showRootWarningDialog = false }
            )
        }
    }
}
