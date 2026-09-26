package com.gemwallet.android.features.settings.presents

import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.settings.viewmodels.SettingsViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.components.screen.rememberSnackbarState
import uniffi.gemstone.GemRowTap

@Composable
fun NotificationsScene(onPriceAlerts: () -> Unit, onCancel: () -> Unit, viewModel: SettingsViewModel = hiltViewModel()) {
    val sections by viewModel.notificationsSections.collectAsStateWithLifecycle()
    val error by viewModel.error.collectAsStateWithLifecycle()
    val snackbar = rememberSnackbarState(message = error, iconRes = R.drawable.ic_error, onShown = viewModel::clearError)

    Scene(
        title = stringResource(id = R.string.settings_notifications_title),
        snackbar = snackbar,
        onClose = onCancel,
    ) {
        LazyColumn {
            sections.forEach { section ->
                itemsPositioned(section.rows) { position, row ->
                    GemListRowView(
                        row = row,
                        listPosition = position,
                        onToggle = { tap, isOn ->
                            when (tap) {
                                GemRowTap.PushNotifications -> if (isOn) viewModel.enableNotifications() else viewModel.disableNotifications()
                                else -> Unit
                            }
                        },
                        onSelect = { tap ->
                            when (tap) {
                                GemRowTap.PriceAlerts -> onPriceAlerts()
                                else -> Unit
                            }
                        },
                    )
                }
            }
        }
    }
}
