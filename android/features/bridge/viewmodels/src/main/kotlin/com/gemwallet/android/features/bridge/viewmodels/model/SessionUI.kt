package com.gemwallet.android.features.bridge.viewmodels.model

import com.gemwallet.android.ext.iconUrl
import com.gemwallet.android.ext.getShortUrl
import com.gemwallet.android.ext.shortName
import com.wallet.core.primitives.ApplicationMetadata

data class SessionUI(
    val icon: String? = null,
    val name: String = "",
    val uri: String = "",
)

fun ApplicationMetadata.toSessionUI(): SessionUI {
    return SessionUI(
        icon = iconUrl,
        name = shortName,
        uri = url.getShortUrl().orEmpty(),
    )
}
