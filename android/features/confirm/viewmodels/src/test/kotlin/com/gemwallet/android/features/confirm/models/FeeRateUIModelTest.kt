package com.gemwallet.android.domains.confirm

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.testkit.mockAssetEthereum
import com.gemwallet.android.testkit.mockAssetPriceInfo
import com.gemwallet.android.testkit.mockAssetPriceValue
import com.gemwallet.android.testkit.mockFormattedNumber
import com.wallet.core.primitives.FeePriority
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.FeeUnitType
import uniffi.gemstone.GemFeeRateRow
import uniffi.gemstone.GemLocalizedText
import uniffi.gemstone.GemNumberUnit
import java.math.BigInteger

class FeeRateUIModelTest {

    @Test
    fun aRateShowsTheScaledFeeInFiat() {
        val model = FeeRateUIModel(
            row = GemFeeRateRow(
                priority = FeePriority.Fast.toGem(),
                fee = BigInteger("500000000000000000"),
                value = GemLocalizedText.FeeRate(mockFormattedNumber(value = 2.5, unit = GemNumberUnit.Plain), FeeUnitType.GWEI),
            ),
            feeAsset = mockAssetPriceValue(mockAssetEthereum(), mockAssetPriceInfo(price = 1.0)),
        )

        assertEquals(FeePriority.Fast, model.priority)
        assertEquals("$0.5", model.fiatValue)
    }

    @Test
    fun aRateWithoutAFeeHasNoFiatValue() {
        val model = FeeRateUIModel(
            row = GemFeeRateRow(
                priority = FeePriority.Normal.toGem(),
                fee = null,
                value = GemLocalizedText.FeeRate(mockFormattedNumber(value = 1.0, unit = GemNumberUnit.Plain), FeeUnitType.NATIVE),
            ),
            feeAsset = mockAssetPriceValue(mockAssetEthereum()),
        )

        assertEquals("", model.fiatValue)
    }
}
