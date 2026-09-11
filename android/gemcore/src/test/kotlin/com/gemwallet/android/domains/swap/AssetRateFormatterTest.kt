package com.gemwallet.android.domains.swap

import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemAssetRate
import uniffi.gemstone.GemSwapRate
import java.util.Locale

class AssetRateFormatterTest {

    private val formatter = AssetRateFormatter(locale = Locale.US)

    @Test
    fun `formats both directions of a core rate like ios`() {
        val pair = formatter.format(
            GemSwapRate(
                direct = GemAssetRate(baseSymbol = "BTC", quoteSymbol = "USDT", value = 100.0),
                inverse = GemAssetRate(baseSymbol = "USDT", quoteSymbol = "BTC", value = 0.01),
            ),
        )

        assertEquals("1 BTC ≈ 100.00 USDT", pair.forward)
        assertEquals("1 USDT ≈ 0.01 BTC", pair.reverse)
        assertEquals("1 BTC ≈ 0.0008382 USDT", formatter.format(GemAssetRate(baseSymbol = "BTC", quoteSymbol = "USDT", value = 0.000838216)))
        assertEquals("1 USDT ≈ 1,193.01 BTC", formatter.format(GemAssetRate(baseSymbol = "USDT", quoteSymbol = "BTC", value = 1193.0109)))
    }

    @Test
    fun `formats tiny swap rates with ios precision`() {
        assertEquals(
            "1 CAKE ≈ 0.00002045 BNB",
            formatter.format(GemAssetRate(baseSymbol = "CAKE", quoteSymbol = "BNB", value = 0.000020446939)),
        )
    }
}
