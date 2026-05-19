package com.gemwallet.android.model

import java.math.BigDecimal
import java.math.BigInteger
import java.math.MathContext

object CryptoFiatConverter {
    fun toFiat(crypto: Crypto, decimals: Int, price: Double): Fiat {
        val result = crypto.atomicValue.toBigDecimal()
            .divide(BigDecimal.TEN.pow(decimals), MathContext.DECIMAL128)
            .multiply(price.toBigDecimal())
        return Fiat(result)
    }

    fun toCrypto(fiat: Fiat, decimals: Int, price: Double): Crypto {
        if (price == 0.0) return Crypto(BigInteger.ZERO)
        val result = fiat.atomicValue
            .divide(price.toBigDecimal(), MathContext.DECIMAL128)
            .multiply(BigDecimal.TEN.pow(decimals))
            .toBigInteger()
        return Crypto(result)
    }
}
