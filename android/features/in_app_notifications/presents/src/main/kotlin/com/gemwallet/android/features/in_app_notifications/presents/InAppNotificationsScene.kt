package com.gemwallet.android.features.in_app_notifications.presents

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.SnackbarHostState
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.features.in_app_notifications.viewmodels.models.InAppNotificationListItemUIModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.empty.EmptyContentType
import com.gemwallet.android.ui.components.empty.EmptyContentView
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.dateSectionedList
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.rememberDateSections
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.models.ListPosition
import uniffi.gemstone.GemEmptyStateKind
import uniffi.gemstone.GemListRow

@Composable
fun InAppNotificationsScene(notifications: List<InAppNotificationListItemUIModel>, errorRow: GemListRow?, snackbar: SnackbarHostState, onAction: (InAppNotificationsAction) -> Unit) {
    Scene(
        title = stringResource(R.string.settings_notifications_title),
        onClose = { onAction(InAppNotificationsAction.Cancel) },
        snackbar = snackbar,
    ) {
        val sections = rememberDateSections(notifications) { it.createdAt }
        if (notifications.isEmpty()) {
            when (val row = errorRow) {
                null -> EmptyContentView(
                    type = EmptyContentType(GemEmptyStateKind.NOTIFICATIONS),
                    modifier = Modifier.fillMaxSize(),
                )

                else -> GemListRowView(row = row, listPosition = ListPosition.Single)
            }
        } else {
            LazyColumn {
                dateSectionedList(
                    sections = sections,
                    key = { _, notification -> notification.id },
                ) { listPosition, notification ->
                    ListItem(
                        model = notification.model,
                        listPosition = listPosition,
                        modifier = notification.destination?.let { destination -> Modifier.clickable { onAction(InAppNotificationsAction.Open(destination)) } } ?: Modifier,
                        accessory = notification.destination?.let { { DataBadgeChevron() } },
                    )
                }
            }
        }
    }
}
