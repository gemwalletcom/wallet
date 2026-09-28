package com.gemwallet.android.features.in_app_notifications.presents

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.SnackbarHostState
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.empty.EmptyContentView
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.ListItemTextStyle
import com.gemwallet.android.ui.components.list_item.dateSectionedList
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.rememberDateSections
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.style.listItemImage
import uniffi.gemstone.GemEmptyStateKind
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemNotificationRow

@Composable
fun InAppNotificationsScene(notifications: List<GemNotificationRow>, errorRow: GemListRow?, snackbar: SnackbarHostState, onAction: (InAppNotificationsAction) -> Unit) {
    Scene(
        title = stringResource(R.string.settings_notifications_title),
        onClose = { onAction(InAppNotificationsAction.Cancel) },
        snackbar = snackbar,
    ) {
        val context = LocalContext.current
        val sections = rememberDateSections(notifications) { it.createdAt }
        if (notifications.isEmpty()) {
            when (val row = errorRow) {
                null -> EmptyContentView(
                    kind = GemEmptyStateKind.NOTIFICATIONS,
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
                        model = ListItemModel(
                            title = notification.title,
                            titleTag = notification.tag?.string(context),
                            titleTagStyle = ListItemTextStyle.Primary,
                            titleExtra = notification.subtitle,
                            subtitle = notification.value,
                            subtitleStyle = ListItemTextStyle.Body,
                            subtitleExtra = notification.subvalue,
                            image = notification.icon?.listItemImage(),
                        ),
                        listPosition = listPosition,
                        modifier = notification.destination?.let { destination -> Modifier.clickable { onAction(InAppNotificationsAction.Open(destination)) } } ?: Modifier,
                        accessory = notification.destination?.let { { DataBadgeChevron() } },
                    )
                }
            }
        }
    }
}
