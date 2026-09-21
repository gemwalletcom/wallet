package com.gemwallet.android.data.coordinators.asset

import com.wallet.core.primitives.Currency
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemValueTone
import uniffi.gemstone.TotalFiatValue as GemTotalFiatValue

class GetWalletSummaryImplTest {

    @Test
    fun walletSummaryEquivalentValue_formatsNegativePercentWithoutSign() {
        val value = WalletSummaryEquivalentValue(
            currency = Currency.USD,
            value = -140.5699884368446,
            changePercentage = -2.84,
        )

        assertEquals("-\$140.57", value.valueFormatted)
        assertEquals("2.84%", value.changePercentageFormatted)
        assertEquals(GemValueTone.NEGATIVE, value.state)
    }

    @Test
    fun walletSummaryEquivalentValue_formatsPositivePercentWithoutSign() {
        val value = WalletSummaryEquivalentValue(
            currency = Currency.USD,
            value = 140.5699884368446,
            changePercentage = 2.84,
        )

        assertEquals("+\$140.57", value.valueFormatted)
        assertEquals("2.84%", value.changePercentageFormatted)
        assertEquals(GemValueTone.POSITIVE, value.state)
    }

    @Test
    fun buildWalletSummaryDisplayState_formatsSmallValuesWithTwoDecimals() {
        val state = buildWalletSummaryDisplayState(
            currency = Currency.USD,
            total = GemTotalFiatValue(value = 0.1041, pnlAmount = 0.1041, pnlPercentage = 0.0),
            showsPnl = true,
        )

        assertEquals("\$0.10", state.totalValue)
        assertEquals("+\$0.10", state.changedValue?.valueFormatted)
    }

    @Test
    fun buildWalletSummaryDisplayState_withZeroBalance_showsZeroTotalAndHidesChange() {
        val state = buildWalletSummaryDisplayState(
            currency = Currency.USD,
            total = GemTotalFiatValue(value = 0.0, pnlAmount = 0.0, pnlPercentage = 0.0),
            showsPnl = false,
        )

        assertEquals("\$0.00", state.totalValue)
        assertEquals(null, state.changedValue)
    }
}
