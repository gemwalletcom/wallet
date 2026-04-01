package com.gemwallet.android.ext

import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemAddressFormatStyle
import uniffi.gemstone.formatAddress

fun String.getAddressEllipsisText(
    chain: Chain? = null,
): String {
    return formatAddress(this, chain?.string, GemAddressFormatStyle.Short)
}
