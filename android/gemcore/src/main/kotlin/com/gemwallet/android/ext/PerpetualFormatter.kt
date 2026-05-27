package com.gemwallet.android.ext

import com.wallet.core.primitives.PerpetualProvider
import uniffi.gemstone.Perpetual
import java.text.DecimalFormatSymbols
import java.util.Locale
import uniffi.gemstone.PerpetualProvider as GemPerpetualProvider

object PerpetualFormatter {

    fun formatPrice(provider: PerpetualProvider, price: Double, decimals: Int): String =
        Perpetual(provider.toGem()).use { it.formatPrice(price, decimals) }

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
        Perpetual(provider.toGem()).use { it.formatSize(size, decimals) }

    fun minimumOrderUsdAmount(provider: PerpetualProvider, price: Double, decimals: Int, leverage: Int): ULong =
        Perpetual(provider.toGem()).use {
            it.minimumOrderUsdAmount(price, decimals, leverage.toUByte())
        }

    private fun PerpetualProvider.toGem(): GemPerpetualProvider = when (this) {
        PerpetualProvider.Hypercore -> GemPerpetualProvider.HYPERCORE
    }
}
