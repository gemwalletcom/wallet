package com.gemwallet.android.ext

import com.wallet.core.primitives.PerpetualId
import com.wallet.core.primitives.PerpetualProvider

fun PerpetualId.toIdentifier(): String = "${provider.string}_$symbol"

fun String.toPerpetualId(): PerpetualId? {
    val separator = indexOf('_').takeIf { it > 0 } ?: return null
    val providerString = substring(0, separator)
    val symbol = substring(separator + 1)
    val provider = PerpetualProvider.entries.firstOrNull { it.string == providerString } ?: return null
    return PerpetualId(provider = provider, symbol = symbol)
}
