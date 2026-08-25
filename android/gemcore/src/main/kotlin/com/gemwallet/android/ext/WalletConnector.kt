package com.gemwallet.android.ext

import com.wallet.core.primitives.Account
import com.wallet.core.primitives.ApplicationMetadata
import com.wallet.core.primitives.ApplicationMetadataSource
import uniffi.gemstone.GemApplicationMetadata
import uniffi.gemstone.GemApplicationMetadataSource
import uniffi.gemstone.applicationMetadataShortName

fun ApplicationMetadata.toGem() = GemApplicationMetadata(
    name = name,
    description = description,
    url = url,
    icon = icon,
    source = source.toGem(),
)

fun GemApplicationMetadata.toPrimitives() = ApplicationMetadata(
    name = name,
    description = description,
    url = url,
    icon = icon,
    source = source.toPrimitives(),
)

fun ApplicationMetadataSource.toGem() = when (this) {
    ApplicationMetadataSource.WalletConnect -> GemApplicationMetadataSource.WALLET_CONNECT
    ApplicationMetadataSource.Payment -> GemApplicationMetadataSource.PAYMENT
}

fun GemApplicationMetadataSource.toPrimitives() = when (this) {
    GemApplicationMetadataSource.WALLET_CONNECT -> ApplicationMetadataSource.WalletConnect
    GemApplicationMetadataSource.PAYMENT -> ApplicationMetadataSource.Payment
}

fun Account.toGem() = uniffi.gemstone.Account(
    chain = chain.string,
    address = address,
    derivationPath = derivationPath,
    extendedPublicKey = extendedPublicKey,
)

val ApplicationMetadata.shortName: String
    get() = applicationMetadataShortName(toGem())

fun List<String>?.walletConnectIcon(): String {
    return this?.firstOrNull { it.endsWith("png", ignoreCase = true) || it.endsWith("jpg", ignoreCase = true) }
        ?: this?.firstOrNull()
        ?: ""
}

fun walletConnectAppName(name: String?, url: String?): String {
    return name?.takeIf { it.isNotBlank() } ?: url?.getShortUrl().orEmpty()
}
