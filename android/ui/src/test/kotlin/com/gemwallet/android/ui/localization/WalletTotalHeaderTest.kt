package com.gemwallet.android.ui.localization

import android.content.Context
import com.gemwallet.android.model.text
import com.wallet.core.primitives.Currency
import io.mockk.mockk
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemCurrencyStyle
import uniffi.gemstone.GemFormattedNumber
import uniffi.gemstone.GemLocalizedText
import uniffi.gemstone.GemPercentageStyle
import uniffi.gemstone.GemValueTone
import uniffi.gemstone.formattedCurrency
import uniffi.gemstone.formattedPercentage
import uniffi.gemstone.formattedSignedCurrency

class WalletTotalHeaderTest {

    private val context = mockk<Context>(relaxed = true)

    @Test
    fun `a small total keeps two decimals and an empty wallet reads as zero`() {
        assertEquals("$0.10", total(0.1041).text())
        assertEquals("$0.00", total(0.0).text())
    }

    @Test
    fun `a change reads as its amount and an unsigned percent`() {
        assertEquals("-$140.57 (2.84%)", change(-140.5699884368446, -2.84).string(context))
        assertEquals("+$140.57 (2.84%)", change(140.5699884368446, 2.84).string(context))
    }

    @Test
    fun `the change takes its tone from the amount`() {
        assertEquals(GemValueTone.NEGATIVE, amount(-140.57).tone)
        assertEquals(GemValueTone.POSITIVE, amount(140.57).tone)
    }

    private fun total(value: Double): GemFormattedNumber = formattedCurrency(value, Currency.USD.string, GemCurrencyStyle.FIAT)

    private fun amount(value: Double): GemFormattedNumber = formattedSignedCurrency(value, Currency.USD.string, GemCurrencyStyle.FIAT)

    private fun change(value: Double, percent: Double): GemLocalizedText = GemLocalizedText.Pnl(amount(value), formattedPercentage(percent, GemPercentageStyle.UNSIGNED))
}
