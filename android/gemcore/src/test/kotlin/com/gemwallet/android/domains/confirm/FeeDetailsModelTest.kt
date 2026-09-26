package com.gemwallet.android.domains.confirm

import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockFeeDetailsModel
import com.gemwallet.android.testkit.mockFeeInfo
import com.gemwallet.android.testkit.mockGemFeeRateRows
import com.wallet.core.primitives.Chain
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.FeeUnitType
import uniffi.gemstone.GemCustomFeeCheck
import uniffi.gemstone.GemLocalizedText
import java.math.BigInteger

class FeeDetailsModelTest {

    @Test
    fun `a custom rate is read by Core against the loaded fee`() {
        val valid = mockFeeDetailsModel().customFee("4")
        assertEquals(BigInteger("4"), valid.rate)
        assertEquals(BigInteger("2000"), valid.feeValue)
        assertNotNull(valid.fee)
        assertTrue(valid.isValid)

        val fractional = mockFeeDetailsModel(rows = mockGemFeeRateRows(unitType = FeeUnitType.GWEI, unitDecimals = 1u, supportsCustomFee = true, selectedTotal = BigInteger("2"), normalTotal = BigInteger("2"))).customFee("0.1")
        assertEquals(BigInteger("1"), fractional.rate)
        assertTrue(fractional.isValid)

        val belowMinimum = mockFeeDetailsModel(
            mockFeeInfo(feeAsset = mockAsset(id = mockAssetId(chain = Chain.Litecoin))),
            mockGemFeeRateRows(unitType = FeeUnitType.GWEI, unitDecimals = 1u, supportsCustomFee = true, selectedTotal = BigInteger("2"), normalTotal = BigInteger("2")),
        ).customFee("0.5")
        val minimum = (belowMinimum.check as GemCustomFeeCheck.BelowMinimum).rate as GemLocalizedText.FeeRate
        assertEquals(5.0, minimum.rate.value, 0.0)
        assertFalse(belowMinimum.isValid)

        val overMax = mockFeeDetailsModel().customFee("21")
        assertTrue(overMax.check is GemCustomFeeCheck.OverMaximum)
        assertFalse(overMax.isValid)

        val anchoredToNormal = mockFeeDetailsModel(rows = mockGemFeeRateRows(unitType = FeeUnitType.GWEI, unitDecimals = 0u, supportsCustomFee = true, selectedTotal = BigInteger("20"), normalTotal = BigInteger("2"))).customFee("21")
        assertTrue(anchoredToNormal.check is GemCustomFeeCheck.OverMaximum)
    }
}
