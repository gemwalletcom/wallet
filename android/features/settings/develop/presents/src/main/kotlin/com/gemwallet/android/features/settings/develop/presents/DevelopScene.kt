package com.gemwallet.android.features.settings.develop.presents

import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.platform.LocalClipboard
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.clipboard.setPlainText
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.features.settings.develop.viewmodels.DevelopViewModel

@Composable
fun DevelopScene(
    onCancel: () -> Unit,
    viewModel: DevelopViewModel = hiltViewModel(),
) {
    val context = LocalContext.current
    val clipboardManager = LocalClipboard.current.nativeClipboard
    val pushToken by viewModel.pushToken.collectAsState()
    Scene(
        title = stringResource(id = R.string.settings_developer),
        onClose = onCancel,
        backHandle = true,
    ) {
        LazyColumn {
            item {
                PropertyItem(
                    "Reset transactions",
                    data = ""
                ) {
                    viewModel.resetTransactions()
                }
            }
            item {
                PropertyItem("Device Id", data = viewModel.getDeviceId()) {
                    clipboardManager.setPlainText(context, viewModel.getDeviceId())
                }
                PropertyItem("Push token", data = pushToken.ifEmpty { "-" }) {
                    clipboardManager.setPlainText(context, pushToken)
                }
                PropertyItem("Store", data = viewModel.platformStore.string) {
                    clipboardManager.setPlainText(context, viewModel.platformStore.string)
                }
            }
        }
    }
}