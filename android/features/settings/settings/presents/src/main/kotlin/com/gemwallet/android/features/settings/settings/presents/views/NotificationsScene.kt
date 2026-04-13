package com.gemwallet.android.features.settings.settings.presents.views

import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.Switch
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.settings.settings.viewmodels.SettingsViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.PushRequest
import com.gemwallet.android.ui.components.list_item.LinkItem
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.models.ListPosition

@Composable
fun NotificationsScene(
    onPriceAlerts: () -> Unit,
    onCancel: () -> Unit,
    viewModel: SettingsViewModel = hiltViewModel(),
) {
    val pushEnabled by viewModel.pushEnabled.collectAsStateWithLifecycle()
    var requestPushGrant by remember { mutableStateOf<(() -> Unit)?>(null) }

    Scene(
        title = stringResource(id = R.string.settings_notifications_title),
        onClose = onCancel,
    ) {
        LazyColumn {
            item {
                LinkItem(
                    title = stringResource(id = R.string.settings_notifications_title),
                    listPosition = ListPosition.Single,
                    trailingContent = @Composable {
                        Switch(
                            checked = pushEnabled,
                            onCheckedChange = {
                                if (it) {
                                    requestPushGrant = viewModel::enableNotifications
                                } else {
                                    viewModel.disableNotifications()
                                }
                            }
                        )
                    },
                    onClick = {}
                )
                LinkItem(
                    title = stringResource(id = R.string.settings_price_alerts_title),
                    icon = R.drawable.settings_pricealert,
                    listPosition = ListPosition.Single,
                    onClick = onPriceAlerts,
                )
            }
        }
    }

    requestPushGrant?.let {
        PushRequest(
            onNotificationEnable = {
                it()
                requestPushGrant = null
            }
        ) { requestPushGrant = null }
    }
}
