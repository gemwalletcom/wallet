package com.gemwallet.android.domains.confirm

import com.gemwallet.android.testkit.mockAmountUIModel
import com.gemwallet.android.testkit.mockAssetPriceInfo
import com.gemwallet.android.testkit.mockAssetPriceValue
import com.gemwallet.android.testkit.mockAssetSolana
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.PaymentPrice
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

    @Test fun formatsCryptoAndEquivalent() {
        assertEquals("1 SOL", mockAmountUIModel().cryptoAmount)
        assertEquals("", mockAmountUIModel().amountEquivalent)
        assertEquals("$200.00", mockAmountUIModel(fromAsset = mockAssetPriceValue(mockAssetSolana(), mockAssetPriceInfo(price = 200.0))).amountEquivalent)
    }

    @Test fun paymentEquivalentIsTheInvoicePrice() {
        assertEquals("$0.1", mockAmountUIModel(paymentPrice = PaymentPrice(currency = "USD", amount = 0.1)).amountEquivalent)
    }
}
