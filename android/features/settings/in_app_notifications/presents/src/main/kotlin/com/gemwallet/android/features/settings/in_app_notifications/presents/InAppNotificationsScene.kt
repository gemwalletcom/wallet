package com.gemwallet.android.features.settings.in_app_notifications.presents

import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.settings.in_app_notifications.presents.components.NotificationItem
import com.gemwallet.android.features.settings.in_app_notifications.viewmodels.InAppNotificationsViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.empty.EmptyContentType
import com.gemwallet.android.ui.components.empty.EmptyContentView
import com.gemwallet.android.ui.components.list_item.dateGroupedList
import com.gemwallet.android.ui.components.screen.Scene

@Composable
fun InAppNotificationsScene(
    onAction: (InAppNotificationsAction) -> Unit,
    viewModel: InAppNotificationsViewModel = hiltViewModel(),
) {
    val notifications by viewModel.notifications.collectAsStateWithLifecycle()

    Scene(
        title = stringResource(R.string.settings_notifications_title),
        onClose = { onAction(InAppNotificationsAction.Cancel) },
    ) {
        if (notifications.isEmpty()) {
            EmptyContentView(
                type = EmptyContentType.Notifications,
                modifier = Modifier.fillMaxSize(),
            )
        } else {
            LazyColumn {
                dateGroupedList(
                    items = notifications,
                    createdAt = { it.createdAt },
                    key = { _, notification -> notification.item.id },
                ) { listPosition, notification ->
                    NotificationItem(
                        notification = notification,
                        listPosition = listPosition,
                        onOpenUrl = { onAction(InAppNotificationsAction.OpenUrl(it)) },
                    )
                }
            }
        }
    }
}
