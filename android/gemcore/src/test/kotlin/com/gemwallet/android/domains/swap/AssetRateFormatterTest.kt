package com.gemwallet.android.domains.swap

import com.gemwallet.android.testkit.mockGemFormattedNumber
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemAssetRate
import uniffi.gemstone.GemNumberDisplay
import uniffi.gemstone.GemNumberUnit
import uniffi.gemstone.GemPrecision
import uniffi.gemstone.GemSwapRate
import java.util.Locale

class AssetRateFormatterTest {

    private val formatter = AssetRateFormatter(locale = Locale.US)

    @Test
    fun `formats both directions of a core rate like ios`() {
        val pair = formatter.format(
            GemSwapRate(
                direct = GemAssetRate(baseSymbol = "BTC", value = places(100.0, "USDT")),
                inverse = GemAssetRate(baseSymbol = "USDT", value = significant(0.01, "BTC")),
            ),
        )

        assertEquals("1 BTC ≈ 100.00 USDT", pair.forward)
        assertEquals("1 USDT ≈ 0.01 BTC", pair.reverse)
        assertEquals("1 BTC ≈ 0.0008382 USDT", formatter.format(GemAssetRate(baseSymbol = "BTC", value = significant(0.000838216, "USDT"))))
        assertEquals("1 USDT ≈ 1,193.01 BTC", formatter.format(GemAssetRate(baseSymbol = "USDT", value = places(1193.0109, "BTC"))))
    }

    @Test
    fun `formats tiny swap rates with ios precision`() {
        assertEquals(
            "1 CAKE ≈ 0.00002045 BNB",
            formatter.format(GemAssetRate(baseSymbol = "CAKE", value = significant(0.000020446939, "BNB"))),
        )
    }

    private fun places(value: Double, symbol: String) = mockGemFormattedNumber(value = value, unit = GemNumberUnit.Symbol(symbol), display = GemNumberDisplay.Number(precision = GemPrecision.Fraction(min = 2u, max = 2u)))

    private fun significant(value: Double, symbol: String) = mockGemFormattedNumber(value = value, unit = GemNumberUnit.Symbol(symbol), display = GemNumberDisplay.Number(precision = GemPrecision.Significant(4u)))
}
