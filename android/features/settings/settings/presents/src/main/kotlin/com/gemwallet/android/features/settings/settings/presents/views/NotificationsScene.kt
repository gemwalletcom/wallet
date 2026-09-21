package com.gemwallet.android.features.settings.settings.presents.views

import androidx.compose.foundation.clickable
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.Switch
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.settings.settings.viewmodels.SettingsViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.components.screen.rememberSnackbarState
import com.gemwallet.android.ui.models.ListPosition

@Composable
fun NotificationsScene(onPriceAlerts: () -> Unit, onCancel: () -> Unit, viewModel: SettingsViewModel = hiltViewModel()) {
    val pushEnabled by viewModel.pushEnabled.collectAsStateWithLifecycle()
    val error by viewModel.error.collectAsStateWithLifecycle()
    val snackbar = rememberSnackbarState(message = error, iconRes = R.drawable.ic_error, onShown = viewModel::clearError)

    Scene(
        title = stringResource(id = R.string.settings_notifications_title),
        snackbar = snackbar,
        onClose = onCancel,
    ) {
        LazyColumn {
            item {
                ListItem(
                    model = viewModel.notificationsListItem,
                    listPosition = ListPosition.Single,
                    accessory = {
                        Switch(
                            checked = pushEnabled,
                            onCheckedChange = {
                                if (it) {
                                    viewModel.enableNotifications()
                                } else {
                                    viewModel.disableNotifications()
                                }
                            },
                        )
                    },
                )
                ListItem(
                    model = viewModel.priceAlertsListItem,
                    listPosition = ListPosition.Single,
                    modifier = Modifier.clickable(onClick = onPriceAlerts),
                    accessory = { DataBadgeChevron() },
                )
            }
        }
    }
}
