package com.gemwallet.android.domains.confirm

import com.gemwallet.android.testkit.mockAssetSolana
import com.gemwallet.android.testkit.mockFeeInfo
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.FeeOption
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

    @Test fun additionalFeesKeepNetworkFeeTotal() {
        val fee = mockFeeInfo(
            amount = BigInteger("1495940"),
            feeAsset = mockAssetSolana(),
            additionalFees = listOf(
                FeeOption.TOKEN_ACCOUNT_CREATION to BigInteger("1488440"),
                FeeOption.TOKEN_ACCOUNT_CREATION to BigInteger("1000"),
            ),
        )

        assertEquals(BigInteger("1495940"), fee.amount)
        assertEquals("0.001495 SOL", fee.cryptoAmount)
        assertEquals(listOf(FeeOption.TOKEN_ACCOUNT_CREATION, FeeOption.TOKEN_ACCOUNT_CREATION), fee.feeItems.map { it.first })
        assertEquals(BigInteger("1488440"), fee.feeItems.first().second.amount)
        assertEquals("0.001488 SOL", fee.feeItems.first().second.cryptoAmount)
        assertEquals(emptyList<Pair<FeeOption, FeeUIModel.FeeInfo>>(), mockFeeInfo(amount = BigInteger("1000000000"), feeAsset = mockAssetSolana()).feeItems)
    }

    @Test fun formatsCryptoAndFiat() {
        val noPrice = mockFeeInfo(amount = BigInteger("1000000000"), feeAsset = mockAssetSolana())
        val withPrice = mockFeeInfo(amount = BigInteger("1000000000"), feeAsset = mockAssetSolana(), price = 200.0)

        assertEquals("1 SOL", noPrice.cryptoAmount)
        assertEquals("", noPrice.fiatAmount)
        assertEquals("$200.00", withPrice.fiatAmount)
    }
}
