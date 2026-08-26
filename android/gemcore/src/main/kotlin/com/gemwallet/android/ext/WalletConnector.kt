package com.gemwallet.android.ext

import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.ApplicationMetadata
import com.wallet.core.primitives.Chain
import uniffi.gemstone.applicationMetadataShortName

fun Account.toGem() = uniffi.gemstone.Account(
    chain = chain.string,
    address = address,
    derivationPath = derivationPath,
    extendedPublicKey = extendedPublicKey,
)

fun uniffi.gemstone.Account.toPrimitives(): Account? {
    val chain = Chain.entries.firstOrNull { it.string == chain } ?: return null
    return Account(
        chain = chain,
        address = address,
        derivationPath = derivationPath,
        extendedPublicKey = extendedPublicKey,
    )
}

val ApplicationMetadata.shortName: String
    get() = applicationMetadataShortName(toJson())

fun List<String>?.walletConnectIcon(): String {
    return this?.firstOrNull { it.endsWith("png", ignoreCase = true) || it.endsWith("jpg", ignoreCase = true) }
        ?: this?.firstOrNull()
        ?: ""
}

fun walletConnectAppName(name: String?, url: String?): String {
    return name?.takeIf { it.isNotBlank() } ?: url?.getShortUrl().orEmpty()
}
