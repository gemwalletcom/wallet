package com.gemwallet.android.application.notifications.cases

interface ShowSystemNotification {
    fun showNotification(
        title: String?,
        subtitle: String?,
        type: String?,
        rawData: String? = null,
    )
}
