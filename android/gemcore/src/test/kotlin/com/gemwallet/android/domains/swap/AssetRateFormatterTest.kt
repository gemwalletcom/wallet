package com.gemwallet.android.domains.swap

import com.gemwallet.android.testkit.mockAsset
import org.junit.Assert.assertEquals
import org.junit.Test
import java.math.BigDecimal
import java.util.Locale

class AssetRateFormatterTest {

    private val formatter = AssetRateFormatter(locale = Locale.US)

    @Test
    fun `formats direct and inverse rates like ios`() {
        val fromAsset = mockAsset(symbol = "BTC")
        val toAsset = mockAsset(symbol = "USDT")

        assertEquals(
            "1 BTC ≈ 100.00 USDT",
            formatter.format(fromAsset, toAsset, BigDecimal("1"), BigDecimal("100")),
        )
        assertEquals(
            "1 USDT ≈ 0.01 BTC",
            formatter.format(fromAsset, toAsset, BigDecimal("1"), BigDecimal("100"), AssetRateFormatter.Direction.Inverse),
        )
        assertEquals(
            "1 BTC ≈ 0.0008382 USDT",
            formatter.format(fromAsset, toAsset, BigDecimal("1312312"), BigDecimal("1100")),
        )
        assertEquals(
            "1 USDT ≈ 1,193.01 BTC",
            formatter.format(fromAsset, toAsset, BigDecimal("1312312"), BigDecimal("1100"), AssetRateFormatter.Direction.Inverse),
        )
    }

    @Test
    fun `formats tiny swap rates with ios precision`() {
        val fromAsset = mockAsset(symbol = "CAKE")
        val toAsset = mockAsset(symbol = "BNB")

        assertEquals(
            "1 CAKE ≈ 0.00002045 BNB",
            formatter.format(fromAsset, toAsset, BigDecimal("1"), BigDecimal("0.000020446939")),
        )
    }
}
