package com.gemwallet.android.features.settings.in_app_notifications.presents

sealed interface InAppNotificationsAction {
    data object Cancel : InAppNotificationsAction
    data class OpenUrl(val url: String) : InAppNotificationsAction
}
