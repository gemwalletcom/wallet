package com.gemwallet.android.domains.confirm

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.model.FeeSelection
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetEthereum
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Chain
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
        GemFeeRate(FeePriority.Normal.toGem(), GemGasPriceType.Regular(gasPrice = "2")),
        GemFeeRate(FeePriority.Fast.toGem(), GemGasPriceType.Regular(gasPrice = "3")),
    )

    private fun currentFee(feeAsset: Asset) = FeeUIModel.FeeInfo(
        amount = BigInteger("1000"),
        feeAsset = feeAsset,
        price = null,
        currency = Currency.USD,
        priority = FeePriority.Normal,
    )

    private fun custom(
        input: String,
        decimals: Int = 0,
        feeAsset: Asset = mockAssetEthereum(),
        selection: FeeSelection = FeeSelection.Preset(FeePriority.Normal),
    ) = CustomFee.from(input, currentFee(feeAsset), feeRates, selection, decimals)

    @Test
    fun customFeeFrom() {
        val valid = custom("4")
        assertEquals(BigInteger("4"), valid.rate)
        assertEquals(BigInteger("2000"), valid.networkFee.amount)
        assertTrue(valid.isConfirmEnabled)

        val fractional = custom("0.1", decimals = 1)
        assertEquals(BigInteger("1"), fractional.rate)
        assertTrue(fractional.isConfirmEnabled)

        val belowMinimum = custom("0.5", decimals = 1, feeAsset = mockAsset(chain = Chain.Litecoin))
        assertTrue(belowMinimum.isBelowMinimum)
        assertEquals("5", belowMinimum.minRateText)
        assertFalse(belowMinimum.isConfirmEnabled)

        val overMax = custom("21")
        assertTrue(overMax.isOverMax)
        assertFalse(overMax.isConfirmEnabled)

        val anchoredToNormal = custom("21", selection = FeeSelection.Custom(BigInteger("20")))
        assertTrue(anchoredToNormal.isOverMax)
    }
}
