package com.gemwallet.android.ext

import com.wallet.core.primitives.PerpetualProvider
import uniffi.gemstone.GemPerpetual
import java.text.DecimalFormatSymbols
import java.util.Locale
import uniffi.gemstone.PerpetualProvider as GemPerpetualProvider

object PerpetualFormatter {

    fun formatPrice(provider: PerpetualProvider, price: Double, decimals: Int): String =
        GemPerpetual(provider.toGemProvider()).use { it.formatPrice(price, decimals) }

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
        GemPerpetual(provider.toGemProvider()).use { it.formatSize(size, decimals) }

    fun minimumOrderUsdAmount(provider: PerpetualProvider, price: Double, decimals: Int, leverage: Int): ULong =
        GemPerpetual(provider.toGemProvider()).use {
            it.minimumOrderUsdAmount(price, decimals, leverage.toUByte())
        }

    fun PerpetualProvider.toGemProvider(): GemPerpetualProvider = when (this) {
        PerpetualProvider.Hypercore -> GemPerpetualProvider.HYPERCORE
    }
}
