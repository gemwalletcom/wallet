package com.gemwallet.android.domains.confirm

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.model.AssetPriceValue
import com.gemwallet.android.testkit.mockAssetEthereum
import com.gemwallet.android.testkit.mockAssetPriceInfo
import com.gemwallet.android.testkit.mockAssetSolana
import com.wallet.core.primitives.FeePriority
import com.wallet.core.primitives.FeeUnitType
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemFeeRateRow
import java.math.BigInteger

class FeeRateUIModelTest {

    @Test
    fun gweiRateShowsTheUnitValueAndTheScaledFeeInFiat() {
        val assetInfo = AssetPriceValue(mockAssetEthereum(), mockAssetPriceInfo(price = 1.0))
        val model = FeeRateUIModel(
            row = GemFeeRateRow(priority = FeePriority.Fast.toGem(), unitValue = BigInteger.ONE, fee = BigInteger("500000000000000000")),
            feeAsset = assetInfo,
            feeUnitType = FeeUnitType.Gwei,
            feeRateDecimals = 9,
            unitSymbol = "gwei",
        )

        assertEquals(FeePriority.Fast, model.priority)
        assertEquals("0.000000001 gwei", model.price)
        assertEquals("$0.5", model.fiatValue)
    }

    @Test
    fun nativeRateShowsTheScaledFeeInTheFeeAsset() {
        val assetInfo = AssetPriceValue(mockAssetSolana(), null)
        fun model(priority: FeePriority, fee: String) = FeeRateUIModel(
            row = GemFeeRateRow(priority = priority.toGem(), unitValue = BigInteger.ONE, fee = BigInteger(fee)),
            feeAsset = assetInfo,
            feeUnitType = FeeUnitType.Native,
            feeRateDecimals = assetInfo.asset.decimals,
            unitSymbol = "SOL",
        )

        assertEquals("0.00011 SOL", model(FeePriority.Normal, "110000").price)
        assertEquals("0.0002 SOL", model(FeePriority.Fast, "200000").price)
    }

    @Test
    fun nativeRateFallsBackToTheUnitValueWithoutAFee() {
        val model = FeeRateUIModel(
            row = GemFeeRateRow(priority = FeePriority.Normal.toGem(), unitValue = BigInteger.ONE, fee = null),
            feeAsset = AssetPriceValue(mockAssetEthereum(), null),
            feeUnitType = FeeUnitType.Native,
            feeRateDecimals = mockAssetEthereum().decimals,
            unitSymbol = "ETH",
        )

        assertEquals("0.000000000000000001 ETH", model.price)
        assertEquals("", model.fiatValue)
    }
}
