package com.gemwallet.android.features.confirm.presents.components

import com.gemwallet.android.domains.confirm.FeeAssetUIModel
import com.gemwallet.android.domains.confirm.FeeDetailsModel
import com.gemwallet.android.domains.confirm.FeeUIModel
import com.gemwallet.android.testkit.mockAssetEthereum
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.FeePriority
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.FeeUnitType
import uniffi.gemstone.GemFeeRateRows
import java.math.BigInteger

class NetworkFeeCustomViewModelTest {

    private val feeAsset = mockAssetEthereum()

    private fun model(decimals: UInt = 0u) = FeeDetailsModel(
        currentFee = FeeUIModel.FeeInfo(
            amount = BigInteger("1000"),
            feeAsset = feeAsset,
            price = null,
            currency = Currency.USD,
            priority = FeePriority.Normal,
        ),
        feeAsset = FeeAssetUIModel(asset = feeAsset, price = null, available = BigInteger("1000000")),
        rows = GemFeeRateRows(
            rows = emptyList(),
            unitType = FeeUnitType.GWEI,
            unitDecimals = decimals,
            supportsCustomFee = true,
            selectedTotal = BigInteger("2"),
            normalTotal = BigInteger("2"),
        ),
    )

    @Test
    fun `the field opens on the rate it was given`() {
        val viewModel = NetworkFeeCustomViewModel(model(), BigInteger("4"))

        assertEquals("4", viewModel.input)
        assertEquals(BigInteger("4"), viewModel.rate)
        assertTrue(viewModel.isConfirmEnabled)
    }

    @Test
    fun `letters never reach the rate`() {
        val viewModel = NetworkFeeCustomViewModel(model(), null)

        viewModel.onInputChange("1a2b")

        assertEquals("12", viewModel.input)
        assertEquals(BigInteger("12"), viewModel.rate)
    }

    @Test
    fun `a rate over the maximum cannot be confirmed`() {
        val viewModel = NetworkFeeCustomViewModel(model(), null)

        viewModel.onInputChange("21")

        assertTrue(viewModel.isOverMax)
        assertFalse(viewModel.isConfirmEnabled)
    }
}
