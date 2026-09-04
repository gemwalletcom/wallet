package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.domains.asset.getIconUrl
import com.gemwallet.android.domains.price.ValueDirection
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.WalletType
import org.junit.Assert.assertEquals
import org.junit.Test
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
        assertEquals(ValueDirection.Down, value.state)
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
        assertEquals(ValueDirection.Up, value.state)
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

    @Test
    fun walletSummaryAggregate_forBaseWallet_usesBaseChainIcon() {
        val summary = WalletSummaryAggregateImpl(
            wallet = mockWallet(
                type = WalletType.Single,
                accounts = listOf(mockAccount(chain = Chain.Base)),
            ),
            displayState = WalletSummaryDisplayState(
                totalValue = "\$0.00",
                changedValue = null,
            ),
            isBalanceHidden = false,
            isOperationsAvailable = true,
        )

        assertEquals(Chain.Base.getIconUrl(), summary.walletIcon.placeholder)
    }
}
