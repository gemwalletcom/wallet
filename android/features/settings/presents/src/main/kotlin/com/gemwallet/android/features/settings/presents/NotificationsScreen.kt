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

    NotificationsScene(sections = sections, snackbar = snackbar) { action ->
        when (action) {
            is NotificationsAction.SetEnabled -> if (action.enabled) viewModel.enableNotifications() else viewModel.disableNotifications()
            NotificationsAction.PriceAlerts -> onPriceAlerts()
            NotificationsAction.Cancel -> onCancel()
        }
    }
}
