package com.gemwallet.android.domains.confirm

import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockAssetSolana
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.TransactionType
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import java.math.BigInteger
import java.util.Locale

class AmountUIModelTest {

    private var originalLocale: Locale = Locale.getDefault()

    @Before fun setUp() {
        originalLocale = Locale.getDefault()
        Locale.setDefault(Locale.US)
    }

    @After fun tearDown() {
        Locale.setDefault(originalLocale)
    }

    private fun model(price: Double?) = AmountUIModel(
        txType = TransactionType.Transfer,
        amount = BigInteger("1000000000"),
        asset = mockAssetInfo(asset = mockAssetSolana()),
        fromAsset = mockAssetInfo(asset = mockAssetSolana()),
        toAsset = null,
        fromAmount = "1000000000",
        toAmount = null,
        nftAsset = null,
        price = price,
        currency = Currency.USD,
    )

    @Test fun formatsCryptoAndEquivalent() {
        assertEquals("1 SOL", model(price = null).cryptoAmount)
        assertEquals("", model(price = null).amountEquivalent)
        assertEquals("$200.00", model(price = 200.0).amountEquivalent)
    }
}
