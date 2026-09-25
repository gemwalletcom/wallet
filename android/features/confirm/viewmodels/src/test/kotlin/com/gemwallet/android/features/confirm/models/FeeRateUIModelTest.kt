package com.gemwallet.android.domains.confirm

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockFormattedNumber
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.FeePriority
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.FeeUnitType
import uniffi.gemstone.GemFeeRateRow
import uniffi.gemstone.GemLocalizedText
import uniffi.gemstone.GemNumberUnit
import uniffi.gemstone.feeAmount
import java.math.BigInteger

class FeeRateUIModelTest {

    @Test
    fun aRateShowsTheScaledFeeInFiat() {
        val model = FeeRateUIModel(
            row = GemFeeRateRow(
                priority = FeePriority.Fast.toGem(),
                fee = BigInteger("500000000000000000"),
                amount = feeAmount(mockAsset(id = mockAssetId(chain = Chain.Ethereum), name = "Ethereum", symbol = "ETH", decimals = 18).toGem(), BigInteger("500000000000000000"), 1.0, Currency.USD.toGem()),
                value = GemLocalizedText.FeeRate(mockFormattedNumber(value = 2.5, unit = GemNumberUnit.Plain), FeeUnitType.GWEI),
                isSelected = false,
            ),
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
                amount = null,
                value = GemLocalizedText.FeeRate(mockFormattedNumber(value = 1.0, unit = GemNumberUnit.Plain), FeeUnitType.NATIVE),
                isSelected = false,
            ),
        )

        assertEquals("", model.fiatValue)
    }
}
