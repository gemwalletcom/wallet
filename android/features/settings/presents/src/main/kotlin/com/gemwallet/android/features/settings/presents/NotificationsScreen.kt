package com.gemwallet.android.features.settings.presents

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.settings.viewmodels.SettingsViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.screen.rememberSnackbarState

@Composable
fun NotificationsScreen(onPriceAlerts: () -> Unit, onCancel: () -> Unit, viewModel: SettingsViewModel = hiltViewModel()) {
    val sections by viewModel.notificationsSections.collectAsStateWithLifecycle()
    val error by viewModel.error.collectAsStateWithLifecycle()
    val snackbar = rememberSnackbarState(message = error, iconRes = R.drawable.ic_error, onShown = viewModel::clearError)

    NotificationsScene(
        sections = sections,
        snackbar = snackbar,
        onEnableNotifications = { viewModel.enableNotifications() },
        onDisableNotifications = { viewModel.disableNotifications() },
        onPriceAlerts = onPriceAlerts,
        onCancel = onCancel,
    )
}
