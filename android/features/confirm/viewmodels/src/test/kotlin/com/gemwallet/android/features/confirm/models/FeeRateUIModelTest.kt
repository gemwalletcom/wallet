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
import uniffi.gemstone.GemFeeRate
import uniffi.gemstone.GemGasPriceType
import java.math.BigInteger

class FeeRateUIModelTest {

    @Test
    fun feeRateScalesFiatFromSelectedLoadedFeeForGweiChain() {
        val assetInfo = AssetPriceValue(mockAssetEthereum(), mockAssetPriceInfo(price = 1.0))
        val model = FeeRateUIModel(
            feeRate = GemFeeRate(FeePriority.Fast.toGem(),
                gasPriceType = GemGasPriceType.Eip1559(gasPrice = BigInteger.ONE, priorityFee = BigInteger.ZERO),
            ),
            feeAsset = assetInfo,
            feeUnitType = FeeUnitType.Gwei,
            feeRateDecimals = 9,
            totalFee = BigInteger("1"),
            selectedTotalFee = BigInteger("2"),
            selectedFeeAmount = BigInteger("1000000000000000000"),
        )

        assertEquals(FeePriority.Fast, model.priority)
        assertEquals("$0.5", model.fiatValue)
    }

    @Test
    fun nativeFeeChainScalesCryptoFromSelectedLoadedFee() {
        val assetInfo = AssetPriceValue(mockAssetSolana(), null)
        fun model(priority: FeePriority, gasPrice: String) = FeeRateUIModel(
            feeRate = GemFeeRate(priority = priority.toGem(), gasPriceType = GemGasPriceType.Regular(gasPrice = BigInteger(gasPrice))),
            feeAsset = assetInfo,
            feeUnitType = FeeUnitType.Native,
            feeRateDecimals = assetInfo.asset.decimals,
            totalFee = BigInteger(gasPrice),
            selectedTotalFee = BigInteger("110"),
            selectedFeeAmount = BigInteger("110000"),
        )

        assertEquals("0.00011 SOL", model(FeePriority.Normal, "110").price)
        assertEquals("0.0002 SOL", model(FeePriority.Fast, "200").price)
    }

    @Test
    fun nativeFeeChainShowsCryptoAmountWithoutFiatWhenFeeNotLoaded() {
        val model = FeeRateUIModel(
            feeRate = GemFeeRate(FeePriority.Normal.toGem(),
                gasPriceType = GemGasPriceType.Regular(gasPrice = BigInteger.ONE),
            ),
            feeAsset = AssetPriceValue(mockAssetEthereum(), null),
            feeUnitType = FeeUnitType.Native,
            feeRateDecimals = mockAssetEthereum().decimals,
            totalFee = BigInteger("1"),
        )

        assertEquals("0.000000000000000001 ETH", model.price)
        assertEquals("", model.fiatValue)
    }
}
