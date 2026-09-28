package com.gemwallet.android.features.in_app_notifications.presents

import uniffi.gemstone.GemNotificationDestination

sealed interface InAppNotificationsAction {
    data object Cancel : InAppNotificationsAction
    data class Open(val destination: GemNotificationDestination) : InAppNotificationsAction
}
