package com.gemwallet.android.features.bridge.viewmodels.model

import com.gemwallet.android.ext.getShortUrl
import com.gemwallet.android.ext.shortName
import com.wallet.core.primitives.WalletConnectionSessionAppMetadata

data class SessionUI(
    val icon: String = "",
    val name: String = "",
    val uri: String = "",
)

fun WalletConnectionSessionAppMetadata.toSessionUI(): SessionUI {
    return SessionUI(
        icon = icon,
        name = shortName,
        uri = url.getShortUrl().orEmpty(),
    )
}
