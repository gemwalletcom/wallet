package com.gemwallet.android.model

import com.wallet.core.primitives.Currency
import java.math.BigDecimal
import java.math.MathContext

object CryptoFiatConverter {
    fun toFiat(crypto: Crypto, decimals: Int, price: Double): Fiat {
        val result = crypto.atomicValue.toBigDecimal()
            .divide(BigDecimal.TEN.pow(decimals), MathContext.DECIMAL128)
            .multiply(price.toBigDecimal())
        return Fiat(result)
    }

    fun toFiatString(crypto: Crypto, decimals: Int, price: Double, currency: Currency): String =
        CurrencyFormatter(currency = currency).string(toFiat(crypto, decimals, price).atomicValue)

    fun toCrypto(fiat: Fiat, decimals: Int, price: Double): Crypto =
        Crypto(cryptoValue(fiat, price), decimals)

    fun toCryptoAtDisplayPrecision(fiat: Fiat, decimals: Int, price: Double): Crypto =
        Crypto(ValueFormatter(style = ValueFormatter.Style.Auto).rounded(cryptoValue(fiat, price)), decimals)

    private fun cryptoValue(fiat: Fiat, price: Double): BigDecimal =
        if (price == 0.0) {
            BigDecimal.ZERO
        } else {
            fiat.atomicValue.divide(price.toBigDecimal(), MathContext.DECIMAL128)
        }
}
