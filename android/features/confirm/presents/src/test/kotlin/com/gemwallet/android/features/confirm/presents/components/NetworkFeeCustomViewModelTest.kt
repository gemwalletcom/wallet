package com.gemwallet.android.features.confirm.presents.components

import com.gemwallet.android.testkit.mockFeeDetailsModel
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import java.math.BigInteger

class NetworkFeeCustomViewModelTest {

    @Test
    fun `the field opens on the rate it was given`() {
        val viewModel = NetworkFeeCustomViewModel(mockFeeDetailsModel(), BigInteger("4"))

        assertEquals("4", viewModel.input)
        assertEquals(BigInteger("4"), viewModel.rate)
        assertTrue(viewModel.isConfirmEnabled)
    }

    @Test
    fun `letters never reach the rate`() {
        val viewModel = NetworkFeeCustomViewModel(mockFeeDetailsModel(), null)

        viewModel.onInputChange("1a2b")

        assertEquals("12", viewModel.input)
        assertEquals(BigInteger("12"), viewModel.rate)
    }

    @Test
    fun `a rate over the maximum cannot be confirmed`() {
        val viewModel = NetworkFeeCustomViewModel(mockFeeDetailsModel(), null)

        viewModel.onInputChange("21")

        assertTrue(viewModel.isOverMax)
        assertFalse(viewModel.isConfirmEnabled)
    }
}
