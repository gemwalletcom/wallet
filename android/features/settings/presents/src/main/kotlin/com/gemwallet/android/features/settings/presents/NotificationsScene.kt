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
import uniffi.gemstone.GemRowTap

@Composable
fun NotificationsScene(sections: List<GemListSection>, snackbar: SnackbarHostState, onEnableNotifications: () -> Unit, onDisableNotifications: () -> Unit, onPriceAlerts: () -> Unit, onCancel: () -> Unit) {
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
                                GemRowTap.PushNotifications -> if (isOn) onEnableNotifications() else onDisableNotifications()
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
