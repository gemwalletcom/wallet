package com.gemwallet.android.ext

import com.wallet.core.primitives.WalletConnectionSessionAppMetadata
import uniffi.gemstone.GemWalletConnectionSessionAppMetadata
import uniffi.gemstone.walletConnectAppShortName

fun WalletConnectionSessionAppMetadata.toGem() = GemWalletConnectionSessionAppMetadata(
    name = name,
    description = description,
    url = url,
    icon = icon,
)

val WalletConnectionSessionAppMetadata.shortName: String
    get() = walletConnectAppShortName(toGem())

fun List<String>?.walletConnectIcon(): String {
    return this?.firstOrNull { it.endsWith("png", ignoreCase = true) || it.endsWith("jpg", ignoreCase = true) }
        ?: this?.firstOrNull()
        ?: ""
}

fun walletConnectAppName(name: String?, url: String?): String {
    return name?.takeIf { it.isNotBlank() } ?: url?.getShortUrl().orEmpty()
}
