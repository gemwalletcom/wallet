package com.gemwallet.android.features.in_app_notifications.presents

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.in_app_notifications.viewmodels.InAppNotificationsViewModel
import com.gemwallet.android.ui.components.screen.rememberSnackbarState
import com.gemwallet.android.ui.models.navigation.RouteMessage

@Composable
fun InAppNotificationsScreen(message: RouteMessage?, onMessageShown: () -> Unit, onAction: (InAppNotificationsAction) -> Unit, viewModel: InAppNotificationsViewModel = hiltViewModel()) {
    val notifications by viewModel.notifications.collectAsStateWithLifecycle()
    val errorRow by viewModel.errorRow.collectAsStateWithLifecycle()
    val snackbar = rememberSnackbarState(message = message, onShown = onMessageShown)

    InAppNotificationsScene(
        notifications = notifications,
        errorRow = errorRow,
        snackbar = snackbar,
        onAction = onAction,
    )
}
