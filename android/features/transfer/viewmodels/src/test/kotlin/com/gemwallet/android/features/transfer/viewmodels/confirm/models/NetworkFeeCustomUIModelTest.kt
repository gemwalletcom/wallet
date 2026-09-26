package com.gemwallet.android.features.transfer.viewmodels.confirm.models

import com.gemwallet.android.testkit.mockFeeDetailsModel
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.GemCustomFeeCheck
import java.math.BigInteger

class NetworkFeeCustomUIModelTest {

    @Test
    fun `the field opens on the rate it was given`() {
        val viewModel = NetworkFeeCustomUIModel(mockFeeDetailsModel(), BigInteger("4"))

        assertEquals("4", viewModel.input)
        assertEquals(BigInteger("4"), viewModel.rate)
        assertTrue(viewModel.isConfirmEnabled)
    }

    @Test
    fun `letters never reach the rate`() {
        val viewModel = NetworkFeeCustomUIModel(mockFeeDetailsModel(), null)

        viewModel.onInputChange("1a2b")

        assertEquals("12", viewModel.input)
        assertEquals(BigInteger("12"), viewModel.rate)
    }

    @Test
    fun `a rate over the maximum cannot be confirmed`() {
        val viewModel = NetworkFeeCustomUIModel(mockFeeDetailsModel(), null)

        viewModel.onInputChange("21")

        assertTrue(viewModel.check is GemCustomFeeCheck.OverMaximum)
        assertFalse(viewModel.isConfirmEnabled)
    }
}
