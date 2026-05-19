package com.gemwallet.android.model

import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import java.math.BigInteger
import java.util.Locale

class ValueFormatterTest {

    @Before
    fun setUp() {
        Locale.setDefault(Locale.US)
    }

    @Test
    fun short() {
        val f = ValueFormatter(style = ValueFormatter.Style.Short, locale = Locale.US)

        assertEquals("0", f.string(BigInteger.ZERO, decimals = 8))
        assertEquals("1 BTC", f.string(BigInteger.valueOf(100_000_000L), decimals = 8, currency = "BTC"))
        assertEquals("82.95 TRX", f.string(BigInteger.valueOf(82_950_000L), decimals = 6, currency = "TRX"))
        assertEquals("8,295 TRX", f.string(BigInteger.valueOf(8_295_000_000L), decimals = 6, currency = "TRX"))
        assertEquals("0.19 HYPE", f.string(BigInteger("199200000000000000"), decimals = 18, currency = "HYPE"))
        assertEquals("0.0999 ETH", f.string(BigInteger.valueOf(99_900L), decimals = 6, currency = "ETH"))
        assertEquals("0.0006 BTC", f.string(BigInteger.valueOf(60_000L), decimals = 8, currency = "BTC"))
        assertEquals("0 BTC", f.string(BigInteger.ONE, decimals = 8, currency = "BTC"))
    }

    @Test
    fun auto() {
        val f = ValueFormatter(style = ValueFormatter.Style.Auto, locale = Locale.US)

        assertEquals("1", f.string(BigInteger("1000000000000000000"), decimals = 18))
        assertEquals("0.1234", f.string(BigInteger("123456789012345678"), decimals = 18))
        assertEquals("0.01234", f.string(BigInteger("12340000000000000"), decimals = 18))
        assertEquals("0.00000546 BTC", f.string(BigInteger.valueOf(546L), decimals = 8, currency = "BTC"))
    }

    @Test
    fun full() {
        val f = ValueFormatter(style = ValueFormatter.Style.Full, locale = Locale.US)

        assertEquals("1 ETH", f.string(BigInteger("1000000000000000000"), decimals = 18, currency = "ETH"))
        assertEquals("1.5 ETH", f.string(BigInteger("1500000000000000000"), decimals = 18, currency = "ETH"))
        assertEquals("1.23456789 ETH", f.string(BigInteger("1234567890000000000"), decimals = 18, currency = "ETH"))
        assertEquals("15,000 USDT", f.string(BigInteger("15000000000000000000000"), decimals = 18, currency = "USDT"))
        assertEquals("0.000000000000000001 ETH", f.string(BigInteger.ONE, decimals = 18, currency = "ETH"))
        assertEquals("0 ETH", f.string(BigInteger.ZERO, decimals = 18, currency = "ETH"))
    }
}
