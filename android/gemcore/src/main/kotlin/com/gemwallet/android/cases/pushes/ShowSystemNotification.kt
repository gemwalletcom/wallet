package com.gemwallet.android.cases.pushes

interface ShowSystemNotification {
    fun showNotification(
        title: String?,
        subtitle: String?,
        type: String?,
        rawData: String? = null,
    )
}
