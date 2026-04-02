package com.gemwallet.android.ext

import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemAddressFormatStyle
import uniffi.gemstone.formatAddress

class AddressFormatter(
    private val address: String,
    private val chain: Chain? = null,
    private val style: Style = Style.Short,
) {
    sealed class Style {
        data object Short : Style()
        data object Full : Style()
        data class Extra(val count: Int) : Style()
    }

    fun value(): String {
        val gemstoneStyle = when (style) {
            is Style.Short -> GemAddressFormatStyle.Short
            is Style.Full -> GemAddressFormatStyle.Full
            is Style.Extra -> GemAddressFormatStyle.Extra(style.count.coerceAtLeast(0).toUInt())
        }
        return formatAddress(address, chain?.string, gemstoneStyle)
    }
}
