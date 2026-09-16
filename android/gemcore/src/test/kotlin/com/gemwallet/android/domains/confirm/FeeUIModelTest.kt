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
import uniffi.gemstone.GemFeeOptionItem
import uniffi.gemstone.FeeOption

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

    @Test fun additionalFeesKeepNetworkFeeTotal() {
        val fee = FeeUIModel.FeeInfo(
            amount = BigInteger("1495940"),
            feeAsset = mockAssetSolana(),
            price = null,
            currency = Currency.USD,
            priority = FeePriority.Normal,
            additionalFees = listOf(
                GemFeeOptionItem(FeeOption.TOKEN_ACCOUNT_CREATION, BigInteger("1488440")),
                GemFeeOptionItem(FeeOption.TOKEN_ACCOUNT_CREATION, BigInteger("1000")),
            ),
        )

        assertEquals(BigInteger("1495940"), fee.amount)
        assertEquals("0.001495 SOL", fee.cryptoAmount)
        assertEquals(listOf(FeeOption.TOKEN_ACCOUNT_CREATION, FeeOption.TOKEN_ACCOUNT_CREATION), fee.feeItems.map { it.first })
        assertEquals(BigInteger("1488440"), fee.feeItems.first().second.amount)
        assertEquals("0.001488 SOL", fee.feeItems.first().second.cryptoAmount)
        assertEquals(emptyList<Pair<FeeOption, FeeUIModel.FeeInfo>>(), feeInfo(price = null).feeItems)
    }

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
