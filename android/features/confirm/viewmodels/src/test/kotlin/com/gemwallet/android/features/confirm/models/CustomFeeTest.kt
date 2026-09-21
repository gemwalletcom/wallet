package com.gemwallet.android.domains.confirm

import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockFeeInfo
import com.gemwallet.android.testkit.mockGemFeeRateRows
import com.wallet.core.primitives.Chain
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.GemCustomFeeCheck
import uniffi.gemstone.GemLocalizedText
import java.math.BigInteger

class CustomFeeTest {

    @Test
    fun customFeeFrom() {
        val valid = CustomFee.from("4", mockFeeInfo(), mockGemFeeRateRows())
        assertEquals(BigInteger("4"), valid.rate)
        assertEquals(BigInteger("2000"), valid.networkFee.amount)
        assertTrue(valid.isConfirmEnabled)

        val fractional = CustomFee.from("0.1", mockFeeInfo(), mockGemFeeRateRows(unitDecimals = 1u))
        assertEquals(BigInteger("1"), fractional.rate)
        assertTrue(fractional.isConfirmEnabled)

        val belowMinimum = CustomFee.from("0.5", mockFeeInfo(feeAsset = mockAsset(chain = Chain.Litecoin)), mockGemFeeRateRows(unitDecimals = 1u))
        val minimum = (belowMinimum.check as GemCustomFeeCheck.BelowMinimum).rate as GemLocalizedText.FeeRate
        assertEquals(5.0, minimum.rate.value, 0.0)
        assertFalse(belowMinimum.isConfirmEnabled)

        val overMax = CustomFee.from("21", mockFeeInfo(), mockGemFeeRateRows())
        assertTrue(overMax.check is GemCustomFeeCheck.OverMaximum)
        assertFalse(overMax.isConfirmEnabled)

        val anchoredToNormal = CustomFee.from("21", mockFeeInfo(), mockGemFeeRateRows(selectedTotal = BigInteger("20")))
        assertTrue(anchoredToNormal.check is GemCustomFeeCheck.OverMaximum)
    }
}
