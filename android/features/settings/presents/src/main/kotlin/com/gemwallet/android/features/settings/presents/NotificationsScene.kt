package com.gemwallet.android.features.settings.presents

import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.SnackbarHostState
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.screen.Scene
import uniffi.gemstone.GemListSection
import uniffi.gemstone.GemRowAction

@Composable
internal fun NotificationsScene(sections: List<GemListSection>, snackbar: SnackbarHostState, onAction: (NotificationsAction) -> Unit) {
    Scene(
        title = stringResource(id = R.string.settings_notifications_title),
        snackbar = snackbar,
        onClose = { onAction(NotificationsAction.Cancel) },
    ) {
        LazyColumn {
            sections.forEach { section ->
                itemsPositioned(section.rows) { position, row ->
                    GemListRowView(
                        row = row,
                        listPosition = position,
                        onToggle = { action, isOn ->
                            when (action) {
                                GemRowAction.PushNotifications -> onAction(NotificationsAction.SetEnabled(isOn))
                                else -> Unit
                            }
                        },
                        onSelect = { action ->
                            when (action) {
                                GemRowAction.PriceAlerts -> onAction(NotificationsAction.PriceAlerts)
                                else -> Unit
                            }
                        },
                    )
                }
            }
        }
    }
}
