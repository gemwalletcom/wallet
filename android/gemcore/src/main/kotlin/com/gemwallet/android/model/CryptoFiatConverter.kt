package com.gemwallet.android.model

import com.wallet.core.primitives.Currency
import java.math.BigDecimal
import uniffi.gemstone.CryptoFiatConverter as GemCryptoFiatConverter

object CryptoFiatConverter {
    private val converter = GemCryptoFiatConverter()

    fun toFiat(crypto: Crypto, decimals: Int, price: Double): Fiat = Fiat(BigDecimal.valueOf(converter.toFiat(crypto.atomicValue, decimals.toUInt(), price)))

    fun fiatValue(crypto: Crypto, decimals: Int, price: Double?): Double? = price?.let { converter.toFiat(crypto.atomicValue, decimals.toUInt(), it) }?.takeIf { it > 0.0 }

    fun toFiatString(crypto: Crypto, decimals: Int, price: Double, currency: Currency): String = CurrencyFormatter(currency = currency).string(toFiat(crypto, decimals, price).atomicValue)
}
