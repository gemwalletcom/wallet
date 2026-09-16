package com.gemwallet.android.model

import com.wallet.core.primitives.Currency
import java.math.BigDecimal
import uniffi.gemstone.CryptoFiatConverter as GemCryptoFiatConverter

object CryptoFiatConverter {
    private val converter = GemCryptoFiatConverter()

    fun toFiat(crypto: Crypto, decimals: Int, price: Double): Fiat =
        Fiat(BigDecimal(converter.toFiat(crypto.atomicValue, decimals.toUInt(), price)))

    fun toFiatString(crypto: Crypto, decimals: Int, price: Double, currency: Currency): String =
        CurrencyFormatter(currency = currency).string(toFiat(crypto, decimals, price).atomicValue)
}
