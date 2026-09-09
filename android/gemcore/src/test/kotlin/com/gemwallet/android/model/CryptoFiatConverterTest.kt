package com.gemwallet.android.model

import org.junit.Assert.assertEquals
import org.junit.Test
import java.math.BigDecimal
import java.math.BigInteger

class CryptoFiatConverterTest {

    @Test
    fun toFiat() {
        assertEquals(
            0,
            CryptoFiatConverter.toFiat(Crypto(BigInteger("100000000")), decimals = 8, price = 50_000.0)
                .atomicValue.compareTo(BigDecimal("50000"))
        )
    }
}
