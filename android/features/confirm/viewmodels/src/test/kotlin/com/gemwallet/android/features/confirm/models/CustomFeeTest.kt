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
import uniffi.gemstone.GemFeeService
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

    private fun custom(input: String, decimals: Int = 0, minimumCustomFeeRate: BigInteger? = null, selection: FeeSelection = FeeSelection.Preset(FeePriority.Normal)) =
        CustomFee.from(input, currentFee, feeRates, selection, decimals, maxMultiplier = 10, minimumCustomFeeRate, GemFeeService())

    @Test
    fun customFeeFrom() {
        val valid = custom("4")
        assertEquals(BigInteger("4"), valid.rate)
        assertEquals(BigInteger("2000"), valid.networkFee.amount)
        assertTrue(valid.isConfirmEnabled)

        val fractional = custom("0.1", decimals = 1, minimumCustomFeeRate = BigInteger.ONE)
        assertEquals(BigInteger("1"), fractional.rate)
        assertTrue(fractional.isConfirmEnabled)

        val belowMinimum = custom("0.3", decimals = 1, minimumCustomFeeRate = BigInteger("5"))
        assertTrue(belowMinimum.isBelowMinimum)
        assertFalse(belowMinimum.isConfirmEnabled)

        val atMinimum = custom("0.5", decimals = 1, minimumCustomFeeRate = BigInteger("5"))
        assertFalse(atMinimum.isBelowMinimum)
        assertTrue(atMinimum.isConfirmEnabled)

        val overMax = custom("21")
        assertTrue(overMax.isOverMax)
        assertFalse(overMax.isConfirmEnabled)

        val anchoredToNormal = custom("21", selection = FeeSelection.Custom(BigInteger("20")))
        assertTrue(anchoredToNormal.isOverMax)
    }
}
