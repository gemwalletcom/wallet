package com.gemwallet.android.ext

import com.wallet.core.primitives.Account
import com.wallet.core.primitives.WalletConnectAppMetadata
import uniffi.gemstone.walletConnectAppShortName

fun Account.toGem() = uniffi.gemstone.Account(
    chain = chain.string,
    address = address,
    derivationPath = derivationPath,
    extendedPublicKey = extendedPublicKey,
)

val WalletConnectAppMetadata.shortName: String
    get() = walletConnectAppShortName(name)

fun List<String>?.walletConnectIcon(): String {
    return this?.firstOrNull { it.endsWith("png", ignoreCase = true) || it.endsWith("jpg", ignoreCase = true) }
        ?: this?.firstOrNull()
        ?: ""
}

fun walletConnectAppName(name: String?, url: String?): String {
    return name?.takeIf { it.isNotBlank() } ?: url?.getShortUrl().orEmpty()
}
