package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.domains.price.PriceState
import com.wallet.core.primitives.Currency
import org.junit.Assert.assertEquals
import org.junit.Test
import java.math.BigDecimal

class GetWalletSummaryImplTest {

    @Test
    fun calculateWalletChangedPercentage_returnsPercentOfTotal() {
        val percentage = calculateWalletChangedPercentage(
            totalValue = BigDecimal("1000.0"),
            changedValue = BigDecimal("-25.0"),
        )

        assertEquals(-2.5, percentage, 0.0)
    }

    @Test
    fun calculateWalletChangedPercentage_withZeroTotal_returnsZero() {
        val percentage = calculateWalletChangedPercentage(
            totalValue = BigDecimal.ZERO,
            changedValue = BigDecimal("-25.0"),
        )

        assertEquals(0.0, percentage, 0.0)
    }

    @Test
    fun walletSummaryEquivalentValue_formatsNegativePercentWithoutSign() {
        val value = WalletSummaryEquivalentValue(
            currency = Currency.USD,
            value = -140.5699884368446,
            changePercentage = -2.84,
        )

        assertEquals("-\$140.56", value.valueFormatted)
        assertEquals("2.84%", value.changePercentageFormatted)
        assertEquals(PriceState.Down, value.state)
    }

    @Test
    fun walletSummaryEquivalentValue_formatsPositivePercentWithoutSign() {
        val value = WalletSummaryEquivalentValue(
            currency = Currency.USD,
            value = 140.5699884368446,
            changePercentage = 2.84,
        )

        assertEquals("\$140.56", value.valueFormatted)
        assertEquals("2.84%", value.changePercentageFormatted)
        assertEquals(PriceState.Up, value.state)
    }

    @Test
    fun buildWalletSummaryDisplayState_withZeroBalance_showsZeroTotalAndHidesChange() {
        val state = buildWalletSummaryDisplayState(
            currency = Currency.USD,
            totalValue = BigDecimal.ZERO,
            totalChangedValue = BigDecimal.ZERO,
            hideBalances = false,
        )

        assertEquals("\$0.00", state.totalValue)
        assertEquals(null, state.changedValue)
    }
}
