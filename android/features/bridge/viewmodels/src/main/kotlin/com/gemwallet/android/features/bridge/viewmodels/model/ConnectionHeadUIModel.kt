package com.gemwallet.android.features.bridge.viewmodels.model

import uniffi.gemstone.GemConnectionRow

data class ConnectionHeadUIModel(
    val iconUrl: String?,
    val title: String,
    val host: String?,
)

fun GemConnectionRow.headUIModel(): ConnectionHeadUIModel = ConnectionHeadUIModel(iconUrl = iconUrl, title = title, host = host)
