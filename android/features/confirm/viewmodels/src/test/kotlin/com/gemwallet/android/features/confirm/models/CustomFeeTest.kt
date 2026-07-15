package com.gemwallet.android.domains.confirm

import com.gemwallet.android.model.FeeSelection
import com.gemwallet.android.testkit.mockAssetEthereum
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.FeePriority
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.GemFeeRate
import uniffi.gemstone.GemGasPriceType
import java.math.BigInteger

class CustomFeeTest {

    private val feeRates = listOf(
        GemFeeRate(FeePriority.Normal.string, GemGasPriceType.Regular(gasPrice = "2")),
        GemFeeRate(FeePriority.Fast.string, GemGasPriceType.Regular(gasPrice = "3")),
    )

    private val currentFee = FeeUIModel.FeeInfo(
        amount = BigInteger("1000"),
        feeAsset = mockAssetEthereum(),
        price = null,
        currency = Currency.USD,
        priority = FeePriority.Normal,
    )

    private fun custom(input: String, selection: FeeSelection) =
        CustomFee.from(input, currentFee, feeRates, selection, decimals = 0, maxMultiplier = 10, unitSymbol = "sat/vB")

    @Test
    fun customFeeComputesRateScalingAndMax() {
        val valid = custom("4", FeeSelection.Preset(FeePriority.Normal))

        assertEquals(BigInteger("4"), valid.rate)
        assertFalse(valid.isOverMax)
        assertTrue(valid.isConfirmEnabled)
        assertEquals(BigInteger("2000"), valid.networkFee.amount)

        val aboveMax = custom("21", FeeSelection.Preset(FeePriority.Normal))

        assertTrue(aboveMax.isOverMax)
        assertFalse(aboveMax.isConfirmEnabled)

        val anchoredToNormal = custom("21", FeeSelection.Custom(BigInteger("20")))

        assertTrue(anchoredToNormal.isOverMax)
    }
}
