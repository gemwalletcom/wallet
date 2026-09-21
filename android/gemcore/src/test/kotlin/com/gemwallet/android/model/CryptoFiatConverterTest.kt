package com.gemwallet.android.model

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import java.math.BigDecimal
import java.math.BigInteger

class CryptoFiatConverterTest {

    @Test
    fun toFiat() {
        assertEquals(
            0,
            CryptoFiatConverter.toFiat(Crypto(BigInteger("100000000")), decimals = 8, price = 50_000.0)
                .atomicValue.compareTo(BigDecimal("50000")),
        )
    }

    @Test
    fun `fiat value is shown only when there is something to show`() {
        val bitcoin = Crypto(BigInteger("100000000"))

        assertEquals(50_000.0, CryptoFiatConverter.fiatValue(bitcoin, decimals = 8, price = 50_000.0)!!, 0.0)
        assertNull(CryptoFiatConverter.fiatValue(bitcoin, decimals = 8, price = null))
        assertNull(CryptoFiatConverter.fiatValue(bitcoin, decimals = 8, price = 0.0))
        assertNull(CryptoFiatConverter.fiatValue(Crypto(BigInteger.ZERO), decimals = 8, price = 50_000.0))
    }
}
