package com.gemwallet.android.domains.confirm

import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockFeeInfo
import com.gemwallet.android.testkit.mockGemFeeRateRows
import com.wallet.core.primitives.Chain
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import java.math.BigInteger

class CustomFeeTest {

    @Test
    fun customFeeFrom() {
        val valid = CustomFee.from("4", mockFeeInfo(), mockGemFeeRateRows(), 0)
        assertEquals(BigInteger("4"), valid.rate)
        assertEquals(BigInteger("2000"), valid.networkFee.amount)
        assertTrue(valid.isConfirmEnabled)

        val fractional = CustomFee.from("0.1", mockFeeInfo(), mockGemFeeRateRows(), 1)
        assertEquals(BigInteger("1"), fractional.rate)
        assertTrue(fractional.isConfirmEnabled)

        val belowMinimum = CustomFee.from("0.5", mockFeeInfo(feeAsset = mockAsset(chain = Chain.Litecoin)), mockGemFeeRateRows(), 1)
        assertTrue(belowMinimum.isBelowMinimum)
        assertEquals("5", belowMinimum.minRateText)
        assertFalse(belowMinimum.isConfirmEnabled)

        val overMax = CustomFee.from("21", mockFeeInfo(), mockGemFeeRateRows(), 0)
        assertTrue(overMax.isOverMax)
        assertFalse(overMax.isConfirmEnabled)

        val anchoredToNormal = CustomFee.from("21", mockFeeInfo(), mockGemFeeRateRows(selectedTotal = BigInteger("20")), 0)
        assertTrue(anchoredToNormal.isOverMax)
    }
}
