package com.gemwallet.android

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.features.support.presents.SupportChatScreen
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.consumeAllPointerEvents
import com.gemwallet.android.ui.components.empty.EmptyAction
import com.gemwallet.android.ui.components.empty.EmptyActionStyle
import com.gemwallet.android.ui.components.empty.EmptyStateView
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.icons.AppIcons

@Composable
internal fun KeystoreKeyChanged(onReset: () -> Unit) {
    var isResetConfirmationShown by rememberSaveable { mutableStateOf(false) }
    var isSupportShown by rememberSaveable { mutableStateOf(false) }

    Box(modifier = Modifier.fillMaxSize().consumeAllPointerEvents()) {
        Scene(title = "") {
            Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                EmptyStateView(
                    title = stringResource(R.string.lock_key_changed_title),
                    description = stringResource(R.string.lock_key_changed_description),
                    iconVector = AppIcons.Warning,
                    buttons = listOf(
                        EmptyAction(title = stringResource(R.string.wallet_import_title), onClick = { isResetConfirmationShown = true }),
                        EmptyAction(title = stringResource(R.string.settings_support), onClick = { isSupportShown = true }, style = EmptyActionStyle.Secondary),
                    ),
                )
            }
        }
        if (isSupportShown) {
            BackHandler { isSupportShown = false }
            SupportChatScreen(message = null, onMessageShown = {}, onCancel = { isSupportShown = false })
        }
    }

    if (isResetConfirmationShown) {
        AlertDialog(
            onDismissRequest = { isResetConfirmationShown = false },
            containerColor = MaterialTheme.colorScheme.background,
            title = { Text(stringResource(R.string.lock_reset_title)) },
            text = {
                Text(
                    text = stringResource(R.string.lock_reset_description),
                    style = MaterialTheme.typography.bodyLarge,
                )
            },
            confirmButton = {
                TextButton(
                    colors = ButtonDefaults.textButtonColors().copy(contentColor = MaterialTheme.colorScheme.error),
                    onClick = {
                        isResetConfirmationShown = false
                        onReset()
                    },
                ) {
                    Text(stringResource(R.string.lock_reset))
                }
            },
            dismissButton = {
                TextButton(onClick = { isResetConfirmationShown = false }) {
                    Text(stringResource(R.string.common_cancel))
                }
            },
        )
    }
}
