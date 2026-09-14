package com.gemwallet.android.ext

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.math.numberFormat
import com.wallet.core.primitives.PerpetualProvider
import uniffi.gemstone.GemPerpetual
import java.util.Locale

object PerpetualFormatter {

    fun formatPrice(provider: PerpetualProvider, price: Double, decimals: Int): String =
        GemPerpetual(provider.toGem()).use { it.formatPrice(price, decimals) }

    fun formatInputPrice(
        provider: PerpetualProvider,
        price: Double,
        decimals: Int,
        locale: Locale = Locale.getDefault(),
    ): String = GemPerpetual(provider.toGem()).use { it.formatInputPrice(price, decimals, numberFormat(locale).decimalSeparator) }
}
