package com.gemwallet.android.domains.confirm

import com.gemwallet.android.testkit.mockAssetSolana
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.FeePriority
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import java.math.BigInteger
import java.util.Locale

class FeeUIModelTest {

    private var originalLocale: Locale = Locale.getDefault()

    @Before fun setUp() {
        originalLocale = Locale.getDefault()
        Locale.setDefault(Locale.US)
    }

    @After fun tearDown() {
        Locale.setDefault(originalLocale)
    }

    private fun feeInfo(price: Double?) = FeeUIModel.FeeInfo(
        amount = BigInteger("1000000000"),
        feeAsset = mockAssetSolana(),
        price = price,
        currency = Currency.USD,
        priority = FeePriority.Normal,
    )

    @Test fun formatsCryptoAndFiat() {
        val noPrice = feeInfo(price = null)
        val withPrice = feeInfo(price = 200.0)

        assertEquals("1 SOL", noPrice.cryptoAmount)
        assertEquals("", noPrice.fiatAmount)
        assertEquals("$200.00", withPrice.fiatAmount)
        assertEquals("1 SOL", noPrice.cryptoAmountWithFiat)
        assertEquals("1 SOL (~$200.00)", withPrice.cryptoAmountWithFiat)
    }
}
