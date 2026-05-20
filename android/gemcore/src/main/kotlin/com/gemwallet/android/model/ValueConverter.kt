package com.gemwallet.android.model

import com.gemwallet.android.math.parseNumber
import java.math.BigDecimal
import java.math.BigInteger
import java.math.MathContext

object ValueConverter {
    fun convertToAmount(fiatValue: String, price: Double, decimals: Int): Crypto {
        if (price == 0.0) return Crypto(BigInteger.ZERO)
        val amount = fiatValue.parseNumber().divide(price.toBigDecimal(), MathContext.DECIMAL128)
        val formatter = ValueFormatter(style = ValueFormatter.Style.Auto)
        val display = formatter.string(Crypto(amount, decimals).atomicValue, decimals)
        return Crypto(display.parseNumber(), decimals)
    }

    fun convertToFiat(amount: String, price: Double): BigDecimal =
        amount.parseNumber().multiply(price.toBigDecimal())
}
