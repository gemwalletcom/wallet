package com.wallet

import com.gemwallet.android.model.AssetBalance
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.compactFormatter
import com.gemwallet.android.model.format
import com.gemwallet.android.model.formatSupply
import com.gemwallet.android.model.shouldUseCompactFormatter
import com.gemwallet.android.model.totalFormatted
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetEthereum
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import junit.framework.TestCase.assertEquals
import org.junit.Test
import java.math.BigInteger
import java.util.Locale

class TestFormat {

    @Test
    fun testFormat_shor() {
        assertEquals(Crypto(BigInteger.valueOf(123)).format(0, "", 2), "123.00")
        assertEquals(Crypto(BigInteger.valueOf(12344)).format(6, "", 2), "0.01")
        assertEquals(Crypto(BigInteger.valueOf(1_000_000)).format(0, "", 2), "1,000,000.00")
        assertEquals(Crypto(BigInteger.valueOf(1_000)).format(0, "", 2), "1,000.00")
        assertEquals(Crypto(BigInteger.valueOf(1)).format(1, "", 2), "0.10")
        assertEquals(Crypto(BigInteger.valueOf(1)).format(2, "", 2), "0.01")
        assertEquals(Crypto(BigInteger.valueOf(1)).format(3, "", 2, dynamicPlace = true), "0.001")
        assertEquals(Crypto(BigInteger.valueOf(1)).format(4, "", 2, dynamicPlace = true), "0.0001")
        assertEquals(Crypto(BigInteger.valueOf(12345678910)).format(6, "", 2), "12,345.67")
        assertEquals(Crypto(BigInteger.valueOf(12345678910)).format(10, "", 2), "1.23")
        assertEquals(Crypto(BigInteger.valueOf(12345678910)).format(18, "", 2, dynamicPlace = true), "0.000000012345")
    }

    @Test
    fun testCurrency_Format() {
        assertEquals(Currency.USD.format(2.0), "$2.00")
        assertEquals(Currency.USD.format(2.04E-6), "$0.00")
        assertEquals(Currency.USD.format(0.0123444, dynamicPlace = true), "$0.01234")
        assertEquals(Currency.USD.format(-0.0123444, dynamicPlace = true), "-\$0.01")
        assertEquals(Currency.USD.format(0.0023444, dynamicPlace = true), "$0.002344")
        assertEquals(Currency.USD.format(-0.0023444, dynamicPlace = true), "-\$0.00")
        assertEquals(Currency.USD.format(0.00023444, dynamicPlace = true), "$0.0002344")
        assertEquals(Currency.USD.format(-0.00023444, dynamicPlace = true), "-\$0.00")
        assertEquals(Currency.USD.format(0.123456, dynamicPlace = true), "$0.1235")
        assertEquals(Currency.USD.format(-0.123456, dynamicPlace = true), "-\$0.12")
        assertEquals(Currency.USD.format(0.00123456, dynamicPlace = true), "$0.001235")
        assertEquals(Currency.USD.format(-0.00123456, dynamicPlace = true), "-\$0.00")
        assertEquals(Currency.USD.format(2.04E-6, dynamicPlace = true), "$0.00000204")
        assertEquals(Currency.USD.format(-2.04E-6, dynamicPlace = true), "-\$0.00")
        assertEquals(Currency.USD.format(4.2795161E-5, dynamicPlace = true), "$0.0000428")
        assertEquals(Currency.USD.format(0.999, dynamicPlace = true), "$1.00")
        assertEquals(Currency.USD.format(1.0E-11, dynamicPlace = true), "$0.00")
        assertEquals(Currency.USD.format(-140.5699884368446, dynamicPlace = true), "-\$140.57")
    }

    @Test
    fun testFormatSupply() {
        val btc = Asset(AssetId(Chain.Bitcoin), "Bitcoin", "BTC", 8, AssetType.NATIVE)
        assertEquals("\u221E BTC", btc.formatSupply(0.0, Locale.US))
    }

    @Test
    fun testCurrency_CompactFormatThreshold() {
        assertEquals(false, shouldUseCompactFormatter(9_999.0))
        assertEquals(true, shouldUseCompactFormatter(10_000.0))
        assertEquals(true, shouldUseCompactFormatter(-10_000.0))
    }

    @Test
    fun testAssetBalance_TotalFormattedTinyBalance() {
        assertEquals(
            "0.0000001 ETH",
            AssetBalance.create(asset = mockAssetEthereum(), available = "100000000000").totalFormatted(),
        )
    }

    @Test
    fun testAsset_CompactFormatBelowThresholdKeepsDefaultFormatting() {
        val asset = mockAsset()

        assertEquals(
            "1.00 BTC",
            asset.compactFormatter(value = 1.0, locale = Locale.US)
        )
    }
}
