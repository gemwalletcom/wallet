package com.gemwallet.android.features.bridge.viewmodels.model

import uniffi.gemstone.GemApplicationMetadataService
import com.gemwallet.android.ext.getShortUrl
import com.gemwallet.android.ext.shortName
import com.wallet.core.primitives.ApplicationMetadata

data class SessionUI(
    val icon: String = "",
    val name: String = "",
    val uri: String = "",
)

fun ApplicationMetadata.toSessionUI(applicationMetadataService: GemApplicationMetadataService): SessionUI {
    return SessionUI(
        icon = icon,
        name = shortName(applicationMetadataService),
        uri = url.getShortUrl().orEmpty(),
    )
}
