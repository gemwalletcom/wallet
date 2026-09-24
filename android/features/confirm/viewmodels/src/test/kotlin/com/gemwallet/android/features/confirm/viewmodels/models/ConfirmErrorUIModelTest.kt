package com.gemwallet.android.features.confirm.viewmodels.models

import android.content.Context
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.testkit.mockAssetEthereum
import io.mockk.every
import io.mockk.mockk
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.Currency
import uniffi.gemstone.GemBalanceRequirement
import uniffi.gemstone.GemConfirmException
import uniffi.gemstone.confirmErrorInfo
import java.math.BigInteger
import java.util.Locale

class ConfirmErrorUIModelTest {
    private val asset = mockAssetEthereum()

    @Before
    fun setUp() {
        Locale.setDefault(Locale.US)
    }

    @Test
    fun aWeiPreciseBalanceReadsAsATruncatedAmount() {
        val error = GemConfirmException.InsufficientBalance(
            asset.toGem(),
            requirement(
                required = "1234567890123456789",
                available = "987654321098765432",
                shortfall = "246913569024691357",
            ),
        )

        assertEquals(
            listOf("**1.23 ETH**", "**0.9876 ETH**", "**0.2469 ETH**"),
            error.sheet()?.descriptionArgs,
        )
    }

    @Test
    fun aBalanceOverAThousandKeepsItsGroupingSeparator() {
        val error = GemConfirmException.InsufficientBalance(
            asset.toGem(),
            requirement(
                required = "12345678901234567890123",
                available = "1",
                shortfall = "12345678901234567890122",
            ),
        )

        assertEquals(
            listOf("**12,345.67 ETH**", "**0.000000000000000001 ETH**", "**12,345.67 ETH**"),
            error.sheet()?.descriptionArgs,
        )
    }

    @Test
    fun aGasSizedNetworkFeeKeepsEveryDigitThatMatters() {
        val error = GemConfirmException.InsufficientNetworkFee(
            asset.toGem(),
            requirement(
                required = "630000000000000",
                available = "500000000000000",
                shortfall = "130000000000000",
            ),
        )

        assertEquals(
            listOf("**0.00063 ETH**", "**Ethereum**", "**0.0005 ETH**", "**0.00013 ETH**"),
            error.sheet()?.descriptionArgs,
        )
    }

    private fun requirement(required: String, available: String, shortfall: String) = GemBalanceRequirement(
        required = BigInteger(required),
        available = BigInteger(available),
        shortfall = BigInteger(shortfall),
    )

    private fun GemConfirmException.sheet() = confirmErrorInfo(this, emptyList(), Currency.USD, "ethereum", "ethereum")?.infoSheet(
        context = mockk<Context> {
            every { getString(any()) } returns "Error"
            every { getString(any(), *anyVararg()) } returns "Error"
        },
        onAcquire = { _, _ -> },
    )
}
