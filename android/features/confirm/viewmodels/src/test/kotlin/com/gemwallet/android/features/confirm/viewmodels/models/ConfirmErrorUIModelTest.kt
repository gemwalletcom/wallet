package com.gemwallet.android.features.confirm.viewmodels.models

import android.content.Context
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.testkit.mockAssetEthereum
import io.mockk.every
import io.mockk.mockk
import java.math.BigInteger
import java.util.Locale
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemAcquireAssetFlow
import uniffi.gemstone.GemBalanceRequirement
import uniffi.gemstone.GemConfirmErrorDisplay

class ConfirmErrorUIModelTest {
    private val asset = mockAssetEthereum()

    @Before
    fun setUp() {
        Locale.setDefault(Locale.US)
    }

    @Test
    fun aWeiPreciseBalanceReadsAsATruncatedAmount() {
        val display = GemConfirmErrorDisplay.BalanceRequired(
            asset.toGem(),
            requirement(
                required = "1234567890123456789",
                available = "987654321098765432",
                shortfall = "246913569024691357",
            ),
        )

        assertEquals(
            listOf("**1.23 ETH**", "**0.9876 ETH**", "**0.2469 ETH**"),
            display.uiModel().info?.descriptionArgs,
        )
    }

    @Test
    fun aBalanceOverAThousandKeepsItsGroupingSeparator() {
        val display = GemConfirmErrorDisplay.BalanceRequired(
            asset.toGem(),
            requirement(
                required = "12345678901234567890123",
                available = "1",
                shortfall = "12345678901234567890122",
            ),
        )

        assertEquals(
            listOf("**12,345.67 ETH**", "**0.000000000000000001 ETH**", "**12,345.67 ETH**"),
            display.uiModel().info?.descriptionArgs,
        )
    }

    @Test
    fun aGasSizedNetworkFeeKeepsEveryDigitThatMatters() {
        val display = GemConfirmErrorDisplay.NetworkFeeRequired(
            asset.toGem(),
            requirement(
                required = "630000000000000",
                available = "500000000000000",
                shortfall = "130000000000000",
            ),
        )

        assertEquals(
            listOf("**0.00063 ETH**", "**Ethereum**", "**0.0005 ETH**", "**0.00013 ETH**"),
            display.uiModel().info?.descriptionArgs,
        )
    }

    private fun requirement(required: String, available: String, shortfall: String) = GemBalanceRequirement(
        required = BigInteger(required),
        available = BigInteger(available),
        shortfall = BigInteger(shortfall),
    )

    private fun GemConfirmErrorDisplay.uiModel(): ConfirmErrorUIModel = uiModel(
        context = mockk<Context> {
            every { getString(any()) } returns "Error"
            every { getString(any(), *anyVararg()) } returns "Error"
        },
        fee = null,
        assetPrice = null,
        networkFeeBuyAmount = 0,
        acquireFlow = { GemAcquireAssetFlow.FIAT },
        onAcquire = { _, _ -> },
    )
}
