package com.gemwallet.android.ext

import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.PerpetualProvider
import uniffi.gemstone.GemPerpetual
import java.text.DecimalFormatSymbols
import java.util.Locale

object PerpetualFormatter {

    fun formatPrice(provider: PerpetualProvider, price: Double, decimals: Int): String =
        GemPerpetual(provider.toGem()).use { it.formatPrice(price, decimals) }

    fun formatInputPrice(
        provider: PerpetualProvider,
        price: Double,
        decimals: Int,
        locale: Locale = Locale.getDefault(),
    ): String {
        val formatted = formatPrice(provider, price, decimals)
        val separator = DecimalFormatSymbols.getInstance(locale).decimalSeparator
        return if (separator == '.') formatted else formatted.replace('.', separator)
    }

    fun formatSize(provider: PerpetualProvider, size: Double, decimals: Int): String =
        GemPerpetual(provider.toGem()).use { it.formatSize(size, decimals) }
}
